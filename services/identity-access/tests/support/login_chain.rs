//! Actual factors + actual mTLS sources + persisted sessions. No positive authority double.
use super::*;
use kitaconnect_identity_access::{
    SessionRecordId,
    login::{self, LoginLimits, LoginRoute, LoginToken},
    mfa::{Enrollment, EphemeralMfaKey, MfaLimits, MfaService, TotpCode},
    password::{AttemptKey, AttemptLimits, Password, PasswordChecker},
    session_check::{self, Requirement},
    session_rotation,
    token::SessionToken,
    webauthn::{CeremonyLimits, LocalhostWebauthn},
};
use webauthn_authenticator_rs::{AuthenticatorBackend, softpasskey::SoftPasskey};
use webauthn_rs::prelude::Url;
use zeroize::Zeroizing;

const BASIC: &str =
    r#"{"format":1,"roles":[{"key":"synthetic_basic","operations":["basic_contact_data"]}]}"#;
const SENSITIVE: &str =
    r#"{"format":1,"roles":[{"key":"synthetic_sensitive","operations":["health_data"]}]}"#;

pub(crate) struct LiveSession {
    token: SessionToken,
    facility: LoginFacilityId,
    reader: JoinedReader,
}
impl LiveSession {
    pub(crate) async fn valid(&mut self) -> bool {
        let mut connection = PgConnection::connect_with(&options("postgres"))
            .await
            .unwrap();
        Box::pin(session_check::check(
            &mut connection,
            &self.token,
            self.facility,
            Requirement::Standard,
            &mut self.reader,
        ))
        .await
        .is_ok()
    }
}
fn user(value: u128) -> IdentityId {
    IdentityId::new(value).unwrap()
}
fn record(value: u128) -> SessionRecordId {
    SessionRecordId::new(value).unwrap()
}
fn login_limits() -> LoginLimits {
    LoginLimits::new(120, 64, 32).unwrap()
}
fn attempt_limits() -> AttemptLimits {
    AttemptLimits::new(3600, 200, 2000, 100).unwrap()
}
fn attempt_key(identity: IdentityId) -> AttemptKey {
    let mut key = [0; 32];
    key[..16].copy_from_slice(&identity.value().to_be_bytes());
    AttemptKey::from_server_key(key)
}
async fn seconds(c: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
fn code(enrollment: &Enrollment, time: u64) -> TotpCode {
    let totp = totp_rs::Builder::new()
        .with_secret(totp_rs::Secret::new_stack(*enrollment.secret.expose()))
        .build()
        .unwrap();
    let raw = Zeroizing::new(totp.generate(time).to_string());
    TotpCode::new(&raw).unwrap()
}
async fn begin(
    c: &mut PgConnection,
    identity: IdentityId,
    site: FacilityId,
    route: LoginRoute,
    reader: &mut JoinedReader,
) -> LoginToken {
    Box::pin(login::begin(
        c,
        identity,
        LoginFacilityId::new(site.value()).unwrap(),
        route,
        login_limits(),
        reader,
    ))
    .await
    .unwrap()
}
async fn prove_password(
    c: &mut PgConnection,
    checker: &PasswordChecker,
    password: &SessionToken,
    identity: IdentityId,
    flow: &LoginToken,
) {
    Box::pin(checker.check_for_login(
        c,
        attempt_key(identity),
        attempt_limits(),
        Some(identity),
        Password::new(password.expose()).unwrap(),
        flow,
    ))
    .await
    .unwrap();
}
async fn prove_passkey(
    c: &mut PgConnection,
    engine: &LocalhostWebauthn,
    client: &mut SoftPasskey,
    identity: IdentityId,
    flow: &LoginToken,
) {
    let (challenge, options) = Box::pin(engine.start_authentication(c, identity))
        .await
        .unwrap();
    let response = client
        .perform_auth(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    Box::pin(engine.finish_authentication_for_login(
        c,
        identity,
        &challenge,
        &serde_json::to_vec(&response).unwrap(),
        flow,
    ))
    .await
    .unwrap();
}
async fn checked(
    c: &mut PgConnection,
    token: &SessionToken,
    site: FacilityId,
    reader: &mut JoinedReader,
) -> bool {
    Box::pin(session_check::check(
        c,
        token,
        LoginFacilityId::new(site.value()).unwrap(),
        Requirement::Standard,
        reader,
    ))
    .await
    .is_ok()
}
async fn state(c: &mut PgConnection, id: u128) -> String {
    sqlx::query_scalar("SELECT state FROM identity_access.sessions WHERE id=$1::text::numeric")
        .bind(id.to_string())
        .fetch_one(c)
        .await
        .unwrap()
}
async fn versions(c: &mut PgConnection, id: u128) -> (String, String, String) {
    sqlx::query_as("SELECT membership_version::text,policy_version::text,rights_version::text FROM identity_access.session_source_observations WHERE session_id=$1::text::numeric")
        .bind(id.to_string()).fetch_one(c).await.unwrap()
}
async fn setup_identity(
    c: &mut PgConnection,
    org: &mut PgConnection,
    identity: IdentityId,
    site: FacilityId,
) {
    sqlx::query("INSERT INTO identity_access.identities VALUES($1::text::numeric)")
        .bind(identity.value().to_string())
        .execute(&mut *c)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organization_config.memberships VALUES($1::text::numeric,$2::text::numeric,1,true)").bind(identity.value().to_string()).bind(site.value().to_string()).execute(org).await.unwrap();
    sqlx::query("INSERT INTO identity_access.role_assurance_scopes VALUES($1::text::numeric,$2::text::numeric,1,'active',$3)")
        .bind(identity.value().to_string()).bind(site.value().to_string()).bind(BASIC).execute(c).await.unwrap();
}
async fn password_session(
    c: &mut PgConnection,
    checker: &PasswordChecker,
    password: &SessionToken,
    identity: IdentityId,
    site: FacilityId,
    reader: &mut JoinedReader,
    id: u128,
) -> SessionToken {
    checker
        .store(c, identity, Password::new(password.expose()).unwrap())
        .await
        .unwrap();
    let flow = begin(c, identity, site, LoginRoute::Password, reader).await;
    prove_password(c, checker, password, identity, &flow).await;
    Box::pin(login::finish(c, &flow, record(id), reader))
        .await
        .unwrap()
}

pub(super) async fn check(
    f: &Fixture<'_, '_>,
    identity_certificate: &Leaf,
    issuer_address: SocketAddr,
    organization_address: SocketAddr,
    parent: FacilityId,
) -> LiveSession {
    let mut c = PgConnection::connect_with(&options("postgres"))
        .await
        .unwrap();
    let mut witness = PgConnection::connect_with(&options("postgres"))
        .await
        .unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut c)
        .await
        .unwrap();
    if parent.value() == 101 {
        assert_eq!(state(&mut c, 10106).await, "revoked");
        assert_eq!(state(&mut c, 10120).await, "active");
        assert_eq!(
            versions(&mut c, 10107).await,
            ("3".into(), "2".into(), "4".into())
        );
        assert_eq!(
            versions(&mut c, 10120).await,
            ("1".into(), "2".into(), "1".into())
        );
    }
    let site = FacilityId::new(parent.value() + 500).unwrap();
    let uid = user(parent.value() + 1000);
    let base = 10000 + parent.value() * 100;
    let mut org = PgConnection::connect_with(&org_options("postgres"))
        .await
        .unwrap();
    postgres::register_facility(&mut org, site, None)
        .await
        .unwrap();
    session_policy::write(&mut org, site, None, SessionPolicyLimits::default())
        .await
        .unwrap();
    setup_identity(&mut c, &mut org, uid, site).await;
    let mut reader = joined(
        f,
        identity_certificate,
        issuer_address,
        organization_address,
        identity_certificate.digest,
        options("postgres"),
    );
    let checker = PasswordChecker::new().await.unwrap();
    let password = SessionToken::generate().unwrap();
    checker
        .store(&mut c, uid, Password::new(password.expose()).unwrap())
        .await
        .unwrap();
    let engine = LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 16).unwrap()).unwrap();
    let mut authenticator = SoftPasskey::new(true);
    let (challenge, registration) = Box::pin(engine.start_registration(&mut c, uid))
        .await
        .unwrap();
    let response = authenticator
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            registration.public_key,
            300_000,
        )
        .unwrap();
    Box::pin(engine.finish_registration(
        &mut c,
        uid,
        &challenge,
        &serde_json::to_vec(&response).unwrap(),
    ))
    .await
    .unwrap();

    // Wrong password cannot create a proof. The correct proof finishes exactly once.
    let flow = begin(&mut c, uid, site, LoginRoute::Password, &mut reader).await;
    let wrong = SessionToken::generate().unwrap();
    assert!(
        Box::pin(checker.check_for_login(
            &mut c,
            attempt_key(uid),
            attempt_limits(),
            Some(uid),
            Password::new(wrong.expose()).unwrap(),
            &flow
        ))
        .await
        .is_err()
    );
    assert!(
        Box::pin(login::finish(&mut c, &flow, record(base), &mut reader))
            .await
            .is_err()
    );
    crate::database_witness::assert_idle(&mut witness, pid).await;
    prove_password(&mut c, &checker, &password, uid, &flow).await;
    let initial = Box::pin(login::finish(&mut c, &flow, record(base), &mut reader))
        .await
        .unwrap();
    assert!(
        Box::pin(login::finish(&mut c, &flow, record(base + 99), &mut reader))
            .await
            .is_err()
    );
    assert!(checked(&mut c, &initial, site, &mut reader).await);
    assert_eq!(
        versions(&mut c, base).await,
        ("1".into(), "1".into(), "1".into())
    );
    let rotated = Box::pin(session_rotation::rotate(
        &mut c,
        &initial,
        record(base + 1),
        LoginFacilityId::new(site.value()).unwrap(),
        &mut reader,
    ))
    .await
    .unwrap();
    assert!(!checked(&mut c, &initial, site, &mut reader).await);
    assert!(checked(&mut c, &rotated, site, &mut reader).await);
    let proof = Box::pin(login::begin_step_up(
        &mut c,
        &rotated,
        LoginFacilityId::new(site.value()).unwrap(),
        LoginRoute::Passkey,
        login_limits(),
        &mut reader,
    ))
    .await
    .unwrap();
    prove_passkey(&mut c, &engine, &mut authenticator, uid, &proof).await;
    let strong = Box::pin(login::finish_step_up(
        &mut c,
        &rotated,
        &proof,
        record(base + 2),
        LoginFacilityId::new(site.value()).unwrap(),
        &mut reader,
    ))
    .await
    .unwrap();
    assert!(!checked(&mut c, &rotated, site, &mut reader).await);
    assert!(checked(&mut c, &strong, site, &mut reader).await);

    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=2 WHERE identity_id=$2::text::numeric AND facility_id=$3::text::numeric")
        .bind(SENSITIVE).bind(uid.value().to_string()).bind(site.value().to_string()).execute(&mut c).await.unwrap();
    sqlx::query("UPDATE organization_config.facility_session_policies SET version=2,idle_seconds=300,absolute_seconds=7200,step_up_seconds=120 WHERE facility_id=$1::text::numeric")
        .bind(site.value().to_string()).execute(&mut org).await.unwrap();
    assert!(checked(&mut c, &strong, site, &mut reader).await);
    assert_eq!(
        versions(&mut c, base + 2).await,
        ("1".into(), "2".into(), "2".into())
    );
    let weak = begin(&mut c, uid, site, LoginRoute::Password, &mut reader).await;
    prove_password(&mut c, &checker, &password, uid, &weak).await;
    assert!(
        Box::pin(login::finish(&mut c, &weak, record(base + 99), &mut reader))
            .await
            .is_err()
    );
    let passkey_flow = begin(&mut c, uid, site, LoginRoute::Passkey, &mut reader).await;
    assert!(
        Box::pin(login::finish(
            &mut c,
            &passkey_flow,
            record(base + 3),
            &mut reader
        ))
        .await
        .is_err()
    );
    prove_passkey(&mut c, &engine, &mut authenticator, uid, &passkey_flow).await;
    let passkey_session = Box::pin(login::finish(
        &mut c,
        &passkey_flow,
        record(base + 3),
        &mut reader,
    ))
    .await
    .unwrap();
    assert!(checked(&mut c, &passkey_session, site, &mut reader).await);
    assert!(
        Box::pin(session_check::check(
            &mut c,
            &passkey_session,
            LoginFacilityId::new(site.value() + 1).unwrap(),
            Requirement::Standard,
            &mut reader
        ))
        .await
        .is_err()
    );

    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=3 WHERE identity_id=$2::text::numeric AND facility_id=$3::text::numeric")
        .bind(r#"{"format":1,"roles":[{"key":"synthetic","operations":["unknown"]}]}"#).bind(uid.value().to_string()).bind(site.value().to_string()).execute(&mut c).await.unwrap();
    assert!(!checked(&mut c, &passkey_session, site, &mut reader).await);
    crate::database_witness::assert_idle(&mut witness, pid).await;
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=4 WHERE identity_id=$2::text::numeric AND facility_id=$3::text::numeric")
        .bind(SENSITIVE).bind(uid.value().to_string()).bind(site.value().to_string()).execute(&mut c).await.unwrap();
    assert!(checked(&mut c, &passkey_session, site, &mut reader).await);
    sqlx::query("UPDATE organization_config.memberships SET active=false,version=2 WHERE identity_ref=$1::text::numeric AND facility_id=$2::text::numeric")
        .bind(uid.value().to_string()).bind(site.value().to_string()).execute(&mut org).await.unwrap();
    assert!(!checked(&mut c, &passkey_session, site, &mut reader).await);
    sqlx::query("UPDATE organization_config.memberships SET active=true,version=3 WHERE identity_ref=$1::text::numeric AND facility_id=$2::text::numeric")
        .bind(uid.value().to_string()).bind(site.value().to_string()).execute(&mut org).await.unwrap();
    assert!(!checked(&mut c, &passkey_session, site, &mut reader).await);
    assert_eq!(state(&mut c, base + 3).await, "revoked");
    let fresh = begin(&mut c, uid, site, LoginRoute::Passkey, &mut reader).await;
    prove_passkey(&mut c, &engine, &mut authenticator, uid, &fresh).await;
    let rejoined = Box::pin(login::finish(&mut c, &fresh, record(base + 5), &mut reader))
        .await
        .unwrap();
    assert!(checked(&mut c, &rejoined, site, &mut reader).await);

    for (pin, database) in [
        (
            identity_certificate.digest,
            options("kc_service_http_runtime"),
        ),
        ([99; 32], options("postgres")),
        (
            identity_certificate.digest,
            options("postgres").database("kc_missing_synthetic_database"),
        ),
    ] {
        let mut failing = joined(
            f,
            identity_certificate,
            issuer_address,
            organization_address,
            pin,
            database,
        );
        assert!(!checked(&mut c, &rejoined, site, &mut failing).await);
        crate::database_witness::assert_idle(&mut witness, pid).await;
        assert!(checked(&mut c, &rejoined, site, &mut reader).await);
    }

    // TOTP uses real advancing steps; no fixture clears a consumed step.
    sqlx::query("INSERT INTO identity_access.mfa_notification_contacts VALUES($1::text::numeric,'00000000-0000-4000-8000-000000000001',0)").bind(uid.value().to_string()).execute(&mut c).await.unwrap();
    let mfa = MfaService::new(
        Arc::new(EphemeralMfaKey::generate().unwrap()),
        MfaLimits::new(300, 300, 4, 1, attempt_limits()).unwrap(),
    );
    let enrollment = Box::pin(mfa.begin_enrollment(&mut c, uid, None))
        .await
        .unwrap();
    let at = seconds(&mut c).await;
    let codes =
        Box::pin(mfa.activate(&mut c, uid, &enrollment, &code(&enrollment, at), None, None))
            .await
            .unwrap();
    let two_factor = begin(&mut c, uid, site, LoginRoute::Password, &mut reader).await;
    prove_password(&mut c, &checker, &password, uid, &two_factor).await;
    assert!(
        Box::pin(login::finish(
            &mut c,
            &two_factor,
            record(base + 6),
            &mut reader
        ))
        .await
        .is_err()
    );
    let at = seconds(&mut c).await + 30;
    let totp = code(&enrollment, at);
    Box::pin(mfa.verify_for_login(&mut c, uid, &totp, &two_factor))
        .await
        .unwrap();
    let totp_session = Box::pin(login::finish(
        &mut c,
        &two_factor,
        record(base + 6),
        &mut reader,
    ))
    .await
    .unwrap();
    assert!(checked(&mut c, &totp_session, site, &mut reader).await);
    assert!(Box::pin(mfa.verify(&mut c, uid, &totp)).await.is_err());
    let permit = Box::pin(mfa.recover(&mut c, uid, &codes.codes()[0]))
        .await
        .unwrap();
    assert!(
        Box::pin(mfa.recover(&mut c, uid, &codes.codes()[0]))
            .await
            .is_err()
    );
    assert!(!checked(&mut c, &totp_session, site, &mut reader).await);
    assert_eq!(state(&mut c, base + 6).await, "revoked");
    assert!(
        Box::pin(login::begin(
            &mut c,
            uid,
            LoginFacilityId::new(site.value()).unwrap(),
            LoginRoute::Password,
            login_limits(),
            &mut reader
        ))
        .await
        .is_err()
    );
    assert!(
        Box::pin(mfa.begin_enrollment(&mut c, uid, None))
            .await
            .is_err()
    );
    let replacement = Box::pin(mfa.begin_enrollment(&mut c, uid, Some(&permit)))
        .await
        .unwrap();
    let at = seconds(&mut c).await;
    Box::pin(mfa.activate(
        &mut c,
        uid,
        &replacement,
        &code(&replacement, at),
        None,
        Some(&permit),
    ))
    .await
    .unwrap();
    assert!(
        Box::pin(mfa.begin_enrollment(&mut c, uid, Some(&permit)))
            .await
            .is_err()
    );
    let restored = begin(&mut c, uid, site, LoginRoute::Password, &mut reader).await;
    prove_password(&mut c, &checker, &password, uid, &restored).await;
    let at = seconds(&mut c).await + 30;
    Box::pin(mfa.verify_for_login(&mut c, uid, &code(&replacement, at), &restored))
        .await
        .unwrap();
    let restored_session = Box::pin(login::finish(
        &mut c,
        &restored,
        record(base + 7),
        &mut reader,
    ))
    .await
    .unwrap();
    assert!(checked(&mut c, &restored_session, site, &mut reader).await);
    assert_eq!(
        versions(&mut c, base + 7).await,
        ("3".into(), "2".into(), "4".into())
    );

    // A separate victim proves role revocation against a previously valid session.
    let victim = user(uid.value() + 2);
    setup_identity(&mut c, &mut org, victim, site).await;
    let victim_token = password_session(
        &mut c,
        &checker,
        &password,
        victim,
        site,
        &mut reader,
        base + 30,
    )
    .await;
    assert!(checked(&mut c, &victim_token, site, &mut reader).await);
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET state='revoked',version=2 WHERE identity_id=$1::text::numeric AND facility_id=$2::text::numeric")
        .bind(victim.value().to_string()).bind(site.value().to_string()).execute(&mut c).await.unwrap();
    assert!(!checked(&mut c, &victim_token, site, &mut reader).await);

    // Keep an independent live identity for the parent's actual service-grant outage/recovery.
    let survivor = user(uid.value() + 1);
    setup_identity(&mut c, &mut org, survivor, site).await;
    let token = password_session(
        &mut c,
        &checker,
        &password,
        survivor,
        site,
        &mut reader,
        base + 20,
    )
    .await;
    assert!(checked(&mut c, &token, site, &mut reader).await);
    assert_eq!(
        versions(&mut c, base + 20).await,
        ("1".into(), "2".into(), "1".into())
    );
    LiveSession {
        token,
        facility: LoginFacilityId::new(site.value()).unwrap(),
        reader,
    }
}
