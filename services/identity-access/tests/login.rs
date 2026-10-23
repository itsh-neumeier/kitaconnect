use kitaconnect_identity_access::{
    IdentityId, SessionRecordId,
    login::{
        self, LoginError, LoginFacilityId, LoginLimits, LoginRoute, LoginToken,
        OrganizationAuthority, OrganizationContext,
    },
    mfa::{Enrollment, EphemeralMfaKey, MfaLimits, MfaService, TotpCode},
    password::{AttemptKey, AttemptLimits, Password, PasswordChecker},
    token::SessionToken,
    webauthn::{CeremonyLimits, LocalhostWebauthn},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use webauthn_authenticator_rs::{AuthenticatorBackend, softpasskey::SoftPasskey};
use webauthn_rs::prelude::*;
use zeroize::Zeroizing;

mod support;

// Explicit service test double: no production membership, network or mTLS claim.
#[derive(Clone)]
struct SyntheticOrganization {
    identity: IdentityId,
    facility: LoginFacilityId,
    version: u64,
    idle: u64,
    absolute: u64,
    step_up: u64,
    mfa: bool,
    denied: bool,
    wrong_identity: bool,
    delay: Duration,
    pause: Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>,
}
impl SyntheticOrganization {
    fn new(identity: IdentityId) -> Self {
        Self {
            identity,
            facility: facility(1),
            version: 1,
            idle: 900,
            absolute: 28_800,
            step_up: 300,
            mfa: false,
            denied: false,
            wrong_identity: false,
            delay: Duration::ZERO,
            pause: None,
        }
    }
}
impl OrganizationAuthority for SyntheticOrganization {
    async fn current(
        &mut self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<OrganizationContext, LoginError> {
        if self.denied || identity != self.identity || facility != self.facility {
            return Err(LoginError);
        }
        tokio::time::sleep(self.delay).await;
        if let Some((entered, release)) = &self.pause {
            entered.notify_one();
            release.notified().await;
        }
        OrganizationContext::new(
            if self.wrong_identity {
                id(u128::MAX)
            } else {
                identity
            },
            facility,
            self.version,
            self.idle,
            self.absolute,
            self.step_up,
            self.mfa,
        )
    }
}
fn id(value: u128) -> IdentityId {
    IdentityId::new(value).unwrap()
}
fn facility(value: u128) -> LoginFacilityId {
    LoginFacilityId::new(value).unwrap()
}
fn session(value: u128) -> SessionRecordId {
    SessionRecordId::new(value).unwrap()
}
fn limits() -> LoginLimits {
    LoginLimits::new(60, 8, 4).unwrap()
}
fn attempts() -> AttemptLimits {
    AttemptLimits::new(3600, 200, 2000, 100).unwrap()
}
fn key(identity: IdentityId) -> AttemptKey {
    let mut value = [0; 32];
    value[..16].copy_from_slice(&identity.value().to_be_bytes());
    AttemptKey::from_server_key(value)
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .username(user)
            .database("kc_login_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn count(c: &mut PgConnection, text: &'static str) -> i64 {
    sqlx::query_scalar(text).fetch_one(c).await.unwrap()
}
async fn clean(c: &mut PgConnection) {
    sql(
        c,
        "DELETE FROM identity_access.login_flows WHERE NOT consumed;",
    )
    .await;
}
async fn time(c: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
async fn force_expired(c: &mut PgConnection, flow: &LoginToken) {
    // Synthetic clock fixture only. Production bindings cannot be changed this way.
    let at = time(c).await;
    let fingerprint = fingerprint_of(c, flow).await;
    sql(
        c,
        "ALTER TABLE identity_access.login_flows DISABLE TRIGGER login_flow_guard;",
    )
    .await;
    sqlx::query("UPDATE identity_access.login_flows SET created_at=$2::text::numeric,expires_at=$3::text::numeric,proven_at=CASE WHEN proof_digest IS NULL THEN NULL ELSE $4::text::numeric END WHERE fingerprint=$1")
        .bind(fingerprint).bind((at-61).to_string()).bind((at-1).to_string()).bind((at-60).to_string()).execute(&mut *c).await.unwrap();
    sql(
        c,
        "ALTER TABLE identity_access.login_flows ENABLE TRIGGER login_flow_guard;",
    )
    .await;
}
async fn cannot_reactivate(
    c: &mut PgConnection,
    flow: &LoginToken,
    org: &mut SyntheticOrganization,
    record: u128,
) {
    let fingerprint = fingerprint_of(c, flow).await;
    let invalidated: bool = sqlx::query_scalar(
        "SELECT invalidated FROM identity_access.login_flows WHERE fingerprint=$1",
    )
    .bind(&fingerprint)
    .fetch_one(&mut *c)
    .await
    .unwrap();
    assert!(invalidated);
    assert!(
        sqlx::query(
            "UPDATE identity_access.login_flows SET invalidated=false WHERE fingerprint=$1"
        )
        .bind(&fingerprint)
        .execute(&mut *c)
        .await
        .is_err()
    );
    // Simulate a deadline becoming future again; the terminal marker must still refuse it.
    sql(
        c,
        "ALTER TABLE identity_access.login_flows DISABLE TRIGGER login_flow_guard;",
    )
    .await;
    sqlx::query(
        "UPDATE identity_access.login_flows SET expires_at=created_at+300 WHERE fingerprint=$1",
    )
    .bind(&fingerprint)
    .execute(&mut *c)
    .await
    .unwrap();
    sql(
        c,
        "ALTER TABLE identity_access.login_flows ENABLE TRIGGER login_flow_guard;",
    )
    .await;
    assert!(login::finish(c, flow, session(record), org).await.is_err());
}
fn code(enrollment: &Enrollment, time: u64) -> TotpCode {
    let totp = totp_rs::Builder::new()
        .with_secret(totp_rs::Secret::new_stack(*enrollment.secret.expose()))
        .build()
        .unwrap();
    let value = Zeroizing::new(totp.generate(time).to_string());
    TotpCode::new(&value).unwrap()
}
async fn prove_password(
    checker: &PasswordChecker,
    c: &mut PgConnection,
    identity: IdentityId,
    password: &SessionToken,
    flow: &LoginToken,
) {
    checker
        .check_for_login(
            c,
            key(identity),
            attempts(),
            Some(identity),
            Password::new(password.expose()).unwrap(),
            flow,
        )
        .await
        .unwrap();
}
async fn begin(
    c: &mut PgConnection,
    identity: IdentityId,
    route: LoginRoute,
    org: &mut SyntheticOrganization,
) -> LoginToken {
    login::begin(c, identity, facility(1), route, limits(), org)
        .await
        .unwrap()
}
async fn wait(c: &mut PgConnection, pid: i32) {
    for _ in 0..200 {
        let waiting: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock')",
        )
        .bind(pid)
        .fetch_one(&mut *c)
        .await
        .unwrap();
        if waiting {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("login contender did not reach its lock");
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}
async fn assertion(
    engine: &LocalhostWebauthn,
    c: &mut PgConnection,
    identity: IdentityId,
    client: &mut SoftPasskey,
) -> (
    kitaconnect_identity_access::webauthn::CeremonyToken,
    Vec<u8>,
) {
    let (token, options) = engine.start_authentication(c, identity).await.unwrap();
    let response = client
        .perform_auth(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    (token, serde_json::to_vec(&response).unwrap())
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn login_completion_is_atomic_and_current() {
    let mut c = connect("postgres").await;
    let connection_pid = pid(&mut c).await;
    let mut cleanup_witness = connect("postgres").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        assert!(count(&mut c,"SELECT count(*) FROM identity_access.login_flows WHERE consumed AND identity_id BETWEEN 1 AND 4").await>=4);
        assert_eq!(count(&mut c,"SELECT count(*) FROM identity_access.session_login_bindings b JOIN identity_access.login_flows f ON b.flow_fingerprint=f.fingerprint WHERE NOT f.consumed").await,0);
        assert!(
            count(
                &mut c,
                "SELECT count(*) FROM identity_access.session_login_bindings WHERE facility_id=1"
            )
            .await
                >= 4
        );
        assert!(
            sqlx::query("UPDATE identity_access.login_flows SET consumed=false WHERE consumed")
                .execute(&mut c)
                .await
                .is_err()
        );
    }
    clean(&mut c).await;
    let base = if after { 100 } else { 0 };
    let a = id(base + 1);
    let b = id(base + 2);
    let d = id(base + 4);
    let user = id(base + 3);
    for identity in [a, b, user, d] {
        sqlx::query("INSERT INTO identity_access.identities VALUES ($1::text::numeric)")
            .bind(identity.value().to_string())
            .execute(&mut c)
            .await
            .unwrap();
    }
    let password = SessionToken::generate().unwrap();
    let checker = PasswordChecker::new().await.unwrap();
    for identity in [a, user, d] {
        checker
            .store(&mut c, identity, Password::new(password.expose()).unwrap())
            .await
            .unwrap();
    }
    let mut a_org = SyntheticOrganization::new(a);
    let mut b_org = SyntheticOrganization::new(b);
    let mut org = SyntheticOrganization::new(user);
    let mut d_org = SyntheticOrganization::new(d);
    // Wrong membership, missing context and a misbound service response fail before creation.
    org.denied = true;
    assert!(
        login::begin(
            &mut c,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    org.denied = false;
    org.wrong_identity = true;
    assert!(
        login::begin(
            &mut c,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    org.wrong_identity = false;
    assert!(
        login::begin(
            &mut c,
            user,
            facility(2),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    let empty = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    let fingerprint: Vec<u8> = sqlx::query_scalar(
        "SELECT fingerprint FROM identity_access.login_flows WHERE identity_id=$1::text::numeric",
    )
    .bind(user.value().to_string())
    .fetch_one(&mut c)
    .await
    .unwrap();
    assert_ne!(fingerprint.as_slice(), empty.expose().as_slice());
    assert!(
        login::finish(&mut c, &empty, session(base + 1000), &mut org)
            .await
            .is_err()
    );
    checker
        .check(
            &mut c,
            key(user),
            attempts(),
            Some(user),
            Password::new(password.expose()).unwrap(),
        )
        .await
        .unwrap();
    assert!(
        login::finish(&mut c, &empty, session(base + 1000), &mut org)
            .await
            .is_err()
    ); // Old evidence is not promoted.
    support::assert_idle(&mut cleanup_witness, connection_pid).await;
    assert!(
        checker
            .check_for_login(
                &mut c,
                key(d),
                attempts(),
                Some(d),
                Password::new(password.expose()).unwrap(),
                &empty
            )
            .await
            .is_err()
    );
    support::assert_idle(&mut cleanup_witness, connection_pid).await;
    let wrong_route = begin(&mut c, user, LoginRoute::Passkey, &mut org).await;
    assert!(
        checker
            .check_for_login(
                &mut c,
                key(user),
                attempts(),
                Some(user),
                Password::new(password.expose()).unwrap(),
                &wrong_route
            )
            .await
            .is_err()
    );
    support::assert_idle(&mut cleanup_witness, connection_pid).await;
    clean(&mut c).await;
    // An enabled TOTP cannot be bypassed even when a synthetic membership does not mandate MFA.
    sqlx::query("INSERT INTO identity_access.mfa_notification_contacts VALUES ($1::text::numeric,'00000000-0000-4000-8000-000000000001',0)").bind(a.value().to_string()).execute(&mut c).await.unwrap();
    let mfa = Arc::new(MfaService::new(
        Arc::new(EphemeralMfaKey::generate().unwrap()),
        MfaLimits::new(300, 300, 4, 1, attempts()).unwrap(),
    ));
    let enrollment = mfa.begin_enrollment(&mut c, a, None).await.unwrap();
    let future = time(&mut c).await + 30;
    let recovery = mfa
        .activate(
            &mut c,
            a,
            &enrollment,
            &code(&enrollment, future),
            None,
            None,
        )
        .await
        .unwrap();
    sqlx::query("UPDATE identity_access.totp_factors SET last_step=NULL WHERE identity_id=$1::text::numeric").bind(a.value().to_string()).execute(&mut c).await.unwrap(); // Fixture only.
    let flow = begin(&mut c, a, LoginRoute::Password, &mut a_org).await;
    prove_password(&checker, &mut c, a, &password, &flow).await;
    assert!(
        login::finish(&mut c, &flow, session(base + 1000), &mut a_org)
            .await
            .is_err()
    );
    mfa.verify_for_login(&mut c, a, &code(&enrollment, future), &flow)
        .await
        .unwrap();
    let issued = login::finish(&mut c, &flow, session(base + 1000), &mut a_org)
        .await
        .unwrap();
    assert!(
        kitaconnect_identity_access::postgres::inspect(&mut c, &issued)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        login::finish(&mut c, &flow, session(base + 1100), &mut a_org)
            .await
            .is_err()
    );
    // Expiration after valid TOTP verification is permanent; no success marker is issued.
    let expired = begin(&mut c, a, LoginRoute::Password, &mut a_org).await;
    prove_password(&checker, &mut c, a, &password, &expired).await;
    force_expired(&mut c, &expired).await;
    sqlx::query("UPDATE identity_access.totp_factors SET last_step=NULL WHERE identity_id=$1::text::numeric").bind(a.value().to_string()).execute(&mut c).await.unwrap();
    let current = time(&mut c).await + 30;
    assert!(
        mfa.verify_for_login(&mut c, a, &code(&enrollment, current), &expired)
            .await
            .is_err()
    );
    let marked: bool =
        sqlx::query_scalar("SELECT totp_ok FROM identity_access.login_flows WHERE fingerprint=$1")
            .bind(fingerprint_of(&mut connect("postgres").await, &expired).await)
            .fetch_one(&mut c)
            .await
            .unwrap();
    assert!(!marked);
    cannot_reactivate(&mut c, &expired, &mut a_org, base + 2200).await;
    clean(&mut c).await;
    // Recovery waits on the identity gate held by completion, then revokes its new session too.
    let stale = begin(&mut c, a, LoginRoute::Password, &mut a_org).await;
    prove_password(&checker, &mut c, a, &password, &stale).await;
    let completing = begin(&mut c, a, LoginRoute::Password, &mut a_org).await;
    prove_password(&checker, &mut c, a, &password, &completing).await;
    sqlx::query("UPDATE identity_access.totp_factors SET last_step=NULL WHERE identity_id=$1::text::numeric")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap(); // Independent race fixture only.
    let future = time(&mut c).await + 30;
    mfa.verify_for_login(&mut c, a, &code(&enrollment, future), &completing)
        .await
        .unwrap();
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mut completing_org = a_org.clone();
    completing_org.pause = Some((entered.clone(), release.clone()));
    let mut completing_connection = connect("postgres").await;
    let completion = tokio::spawn(async move {
        login::finish(
            &mut completing_connection,
            &completing,
            session(base + 2000),
            &mut completing_org,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(10), entered.notified())
        .await
        .unwrap();
    let mut recovery_connection = connect("postgres").await;
    let recovery_pid = pid(&mut recovery_connection).await;
    let recovery_engine = mfa.clone();
    let recovery_code =
        kitaconnect_identity_access::mfa::RecoveryCode::from_bytes(recovery.codes()[0].expose())
            .unwrap();
    let recovery_task = tokio::spawn(async move {
        recovery_engine
            .recover(&mut recovery_connection, a, &recovery_code)
            .await
    });
    wait(&mut c, recovery_pid).await;
    release.notify_one();
    let just_issued = completion.await.unwrap().unwrap();
    recovery_task.await.unwrap().unwrap();
    assert!(
        kitaconnect_identity_access::postgres::inspect(&mut c, &just_issued)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        login::finish(&mut c, &stale, session(base + 1100), &mut a_org)
            .await
            .is_err()
    );
    assert!(
        login::begin(
            &mut c,
            a,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut a_org
        )
        .await
        .is_err()
    );
    assert!(
        kitaconnect_identity_access::postgres::inspect(&mut c, &issued)
            .await
            .unwrap()
            .is_none()
    );
    clean(&mut c).await;
    // Passkey with real library signatures and UV fulfills the confirmed MFA alternative.
    let engine = LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 8).unwrap()).unwrap();
    let mut client = SoftPasskey::new(true);
    let (ceremony, options) = engine.start_registration(&mut c, b).await.unwrap();
    let response = client
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    let credential = engine
        .finish_registration(
            &mut c,
            b,
            &ceremony,
            &serde_json::to_vec(&response).unwrap(),
        )
        .await
        .unwrap();
    b_org.mfa = true;
    let expired = begin(&mut c, b, LoginRoute::Passkey, &mut b_org).await;
    force_expired(&mut c, &expired).await;
    let (challenge, response) = assertion(&engine, &mut c, b, &mut client).await;
    assert!(
        engine
            .finish_authentication_for_login(&mut c, b, &challenge, &response, &expired)
            .await
            .is_err()
    );
    cannot_reactivate(&mut c, &expired, &mut b_org, base + 2201).await;
    clean(&mut c).await;
    let flow = begin(&mut c, b, LoginRoute::Passkey, &mut b_org).await;
    let (challenge, response) = assertion(&engine, &mut c, b, &mut client).await;
    engine
        .finish_authentication_for_login(&mut c, b, &challenge, &response, &flow)
        .await
        .unwrap();
    let issued = login::finish(&mut c, &flow, session(base + 1001), &mut b_org)
        .await
        .unwrap();
    assert!(
        engine
            .finish_authentication_for_login(&mut c, b, &challenge, &response, &flow)
            .await
            .is_err()
    );
    let stale = begin(&mut c, b, LoginRoute::Passkey, &mut b_org).await;
    let (challenge, response) = assertion(&engine, &mut c, b, &mut client).await;
    engine
        .finish_authentication_for_login(&mut c, b, &challenge, &response, &stale)
        .await
        .unwrap();
    assert!(engine.revoke(&mut c, b, &credential).await.unwrap());
    assert!(
        login::finish(&mut c, &stale, session(base + 1101), &mut b_org)
            .await
            .is_err()
    );
    assert!(
        kitaconnect_identity_access::postgres::inspect(&mut c, &issued)
            .await
            .unwrap()
            .is_none()
    );
    clean(&mut c).await;
    // Actual waiters: one flow can create one session, with the now stricter policy.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &flow).await;
    org.version = 2;
    org.idle = 120;
    org.absolute = 3600;
    org.step_up = 60;
    let mut guard = connect("postgres").await;
    let mut tx = guard.begin().await.unwrap();
    sqlx::query("SELECT FROM identity_access.login_flows WHERE fingerprint=$1 FOR UPDATE")
        .bind(fingerprint_of(&mut c, &flow).await)
        .execute(&mut *tx)
        .await
        .unwrap();
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let lp = pid(&mut left).await;
    let rp = pid(&mut right).await;
    let l_token = LoginToken::from_bytes(flow.expose()).unwrap();
    let r_token = LoginToken::from_bytes(flow.expose()).unwrap();
    let mut l_org = org.clone();
    let mut r_org = org.clone();
    let l = tokio::spawn(async move {
        login::finish(&mut left, &l_token, session(base + 1002), &mut l_org).await
    });
    let r = tokio::spawn(async move {
        login::finish(&mut right, &r_token, session(base + 1003), &mut r_org).await
    });
    wait(&mut c, lp).await;
    wait(&mut c, rp).await;
    tx.commit().await.unwrap();
    assert_eq!(
        usize::from(l.await.unwrap().is_ok()) + usize::from(r.await.unwrap().is_ok()),
        1
    );
    let (idle,absolute,step,version,mfa):(String,String,String,String,bool)=sqlx::query_as("SELECT s.idle_seconds::text,(s.absolute_expires_at-s.created_at)::text,b.step_up_seconds::text,b.policy_version::text,b.mfa FROM identity_access.sessions s JOIN identity_access.session_login_bindings b ON b.session_id=s.id WHERE s.identity_id=$1::text::numeric")
        .bind(user.value().to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(
        (idle, absolute, step, version, mfa),
        ("120".into(), "3600".into(), "60".into(), "2".into(), false)
    );
    // Current membership and policy are re-read; equal-version mutation and downgrade fail.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &flow).await;
    org.denied = true;
    assert!(
        login::finish(&mut c, &flow, session(base + 1004), &mut org)
            .await
            .is_err()
    );
    org.denied = false;
    org.idle = 100;
    assert!(
        login::finish(&mut c, &flow, session(base + 1004), &mut org)
            .await
            .is_err()
    );
    org.idle = 120;
    org.version = 1;
    assert!(
        login::finish(&mut c, &flow, session(base + 1004), &mut org)
            .await
            .is_err()
    );
    org.version = 2;
    org.mfa = true;
    assert!(
        login::finish(&mut c, &flow, session(base + 1004), &mut org)
            .await
            .is_err()
    );
    org.mfa = false;
    org.version = 3;
    org.idle = 900;
    org.absolute = 28_800;
    org.step_up = 300;
    login::finish(&mut c, &flow, session(base + 1004), &mut org)
        .await
        .unwrap();
    let idle: String = sqlx::query_scalar(
        "SELECT idle_seconds::text FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 1004).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap();
    assert_eq!(idle, "120");
    d_org.mfa = true;
    let required = begin(&mut c, d, LoginRoute::Password, &mut d_org).await;
    prove_password(&checker, &mut c, d, &password, &required).await;
    d_org.mfa = false;
    assert!(
        login::finish(&mut c, &required, session(base + 1104), &mut d_org)
            .await
            .is_err()
    );
    clean(&mut c).await;
    // Password replacement and PHC mutation without revision advancement invalidate proofs.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &flow).await;
    let phc: String = sqlx::query_scalar(
        "SELECT phc FROM identity_access.password_credentials WHERE identity_id=$1::text::numeric",
    )
    .bind(user.value().to_string())
    .fetch_one(&mut c)
    .await
    .unwrap();
    sqlx::query("UPDATE identity_access.password_credentials SET phc='fixture-invalid' WHERE identity_id=$1::text::numeric").bind(user.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        login::finish(&mut c, &flow, session(base + 1105), &mut org)
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.password_credentials SET phc=$2 WHERE identity_id=$1::text::numeric").bind(user.value().to_string()).bind(phc).execute(&mut c).await.unwrap();
    checker
        .store(&mut c, user, Password::new(password.expose()).unwrap())
        .await
        .unwrap();
    assert!(
        login::finish(&mut c, &flow, session(base + 1105), &mut org)
            .await
            .is_err()
    );
    clean(&mut c).await;
    // A failure after flow consumption must roll back every local write.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &flow).await;
    sql(&mut c,"CREATE FUNCTION public.kc_login_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture commit failure' USING ERRCODE='23514'; END $$; CREATE CONSTRAINT TRIGGER kc_login_fail AFTER INSERT ON identity_access.session_login_bindings DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.kc_login_fail();").await;
    assert!(
        login::finish(&mut c, &flow, session(base + 1005), &mut org)
            .await
            .is_err()
    );
    sql(&mut c,"DROP TRIGGER kc_login_fail ON identity_access.session_login_bindings; DROP FUNCTION public.kc_login_fail(); CREATE FUNCTION public.kc_login_suppress() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER kc_login_suppress BEFORE INSERT ON identity_access.session_login_bindings FOR EACH ROW EXECUTE FUNCTION public.kc_login_suppress();").await;
    assert!(
        login::finish(&mut c, &flow, session(base + 1005), &mut org)
            .await
            .is_err()
    );
    sql(&mut c,"DROP TRIGGER kc_login_suppress ON identity_access.session_login_bindings; DROP FUNCTION public.kc_login_suppress();").await;
    let before: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 1005).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap();
    assert_eq!(before, 0);
    login::finish(&mut c, &flow, session(base + 1005), &mut org)
        .await
        .unwrap();
    assert!(
        login::finish(&mut c, &flow, session(base + 1105), &mut org)
            .await
            .is_err()
    );
    // Valid password verification of an expired flow also persists its terminal state.
    let expired = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    force_expired(&mut c, &expired).await;
    assert!(
        checker
            .check_for_login(
                &mut c,
                key(user),
                attempts(),
                Some(user),
                Password::new(password.expose()).unwrap(),
                &expired
            )
            .await
            .is_err()
    );
    cannot_reactivate(&mut c, &expired, &mut org, base + 2202).await;
    clean(&mut c).await;
    let expired = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &expired).await;
    force_expired(&mut c, &expired).await;
    assert!(
        login::finish(&mut c, &expired, session(base + 2203), &mut org)
            .await
            .is_err()
    );
    cannot_reactivate(&mut c, &expired, &mut org, base + 2203).await;
    clean(&mut c).await;
    // Expiration while waiting on the external authority is rechecked after the wait.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    prove_password(&checker, &mut c, user, &password, &flow).await;
    sql(
        &mut c,
        "ALTER TABLE identity_access.login_flows DISABLE TRIGGER login_flow_guard;",
    )
    .await;
    let at = time(&mut c).await;
    sqlx::query("UPDATE identity_access.login_flows SET created_at=$2::text::numeric,expires_at=$3::text::numeric WHERE fingerprint=$1")
        .bind(fingerprint_of(&mut connect("postgres").await,&flow).await).bind((at-58).to_string()).bind((at+2).to_string()).execute(&mut c).await.unwrap();
    sql(
        &mut c,
        "ALTER TABLE identity_access.login_flows ENABLE TRIGGER login_flow_guard;",
    )
    .await;
    org.delay = Duration::from_secs(3);
    assert!(
        login::finish(&mut c, &flow, session(base + 1106), &mut org)
            .await
            .is_err()
    );
    org.delay = Duration::ZERO;
    cannot_reactivate(&mut c, &flow, &mut org, base + 2204).await;
    clean(&mut c).await;
    // Shared limits are immutable to callers; quotas count pending flows only.
    assert!(
        login::begin(
            &mut c,
            user,
            facility(1),
            LoginRoute::Password,
            LoginLimits::new(61, 8, 4).unwrap(),
            &mut org
        )
        .await
        .is_err()
    );
    for _ in 0..4 {
        begin(&mut c, user, LoginRoute::Password, &mut org).await;
    }
    assert!(
        login::begin(
            &mut c,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    for _ in 0..4 {
        begin(&mut c, d, LoginRoute::Password, &mut d_org).await;
    }
    assert!(
        login::begin(
            &mut c,
            b,
            facility(1),
            LoginRoute::Passkey,
            limits(),
            &mut b_org
        )
        .await
        .is_err()
    );
    clean(&mut c).await;
    sql(
        &mut c,
        "UPDATE identity_access.login_limits SET last_observed=18446744073709551615;",
    )
    .await;
    assert!(
        login::begin(
            &mut c,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut cleanup_witness, connection_pid).await;
    sql(
        &mut c,
        "UPDATE identity_access.login_limits SET last_observed=0;",
    )
    .await; // Isolated test setup only.
    let flow = begin(&mut c, user, LoginRoute::Password, &mut org).await;
    let mut tx = c.begin().await.unwrap();
    assert!(
        login::finish(&mut tx, &flow, session(base + 1107), &mut org)
            .await
            .is_err()
    );
    assert!(
        login::begin(
            &mut tx,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    let mut runtime = connect("kc_identity_test_runtime").await;
    assert!(
        login::begin(
            &mut runtime,
            user,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    assert!(
        login::finish(&mut runtime, &flow, session(base + 1107), &mut org)
            .await
            .is_err()
    );
    for table in ["login_flows", "login_limits", "session_login_bindings"] {
        let statement =
            sqlx::AssertSqlSafe(format!("SELECT count(*) FROM identity_access.{table}"));
        let visible: i64 = sqlx::query_scalar(statement)
            .fetch_one(&mut runtime)
            .await
            .unwrap();
        assert_eq!(visible, 0);
        let mut tx = c.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE kc_identity_owner")
            .execute(&mut *tx)
            .await
            .unwrap();
        let visible: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM identity_access.{table}"
        )))
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(visible, 0);
        tx.rollback().await.unwrap();
    }
    assert_eq!(
        count(
            &mut c,
            "SELECT count(*) FROM pg_policies WHERE schemaname='identity_access'"
        )
        .await,
        0
    );
}
async fn fingerprint_of(c: &mut PgConnection, flow: &LoginToken) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let fingerprint: Vec<u8> = Sha256::digest(flow.expose()).to_vec();
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM identity_access.login_flows WHERE fingerprint=$1)",
    )
    .bind(&fingerprint)
    .fetch_one(c)
    .await
    .unwrap();
    assert!(exists);
    fingerprint
}
