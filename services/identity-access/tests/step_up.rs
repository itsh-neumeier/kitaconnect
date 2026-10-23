use kitaconnect_identity_access::{
    IdentityId, SessionRecordId,
    login::{
        self, LoginError, LoginFacilityId, LoginLimits, LoginRoute, LoginToken,
        OrganizationAuthority, OrganizationContext,
    },
    mfa::{Enrollment, EphemeralMfaKey, MfaLimits, MfaService, TotpCode},
    password::{AttemptKey, AttemptLimits, Password, PasswordChecker},
    session_check::{self, Requirement},
    session_rotation,
    token::SessionToken,
    webauthn::{CeremonyLimits, LocalhostWebauthn},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use webauthn_authenticator_rs::{AuthenticatorBackend, softpasskey::SoftPasskey};
use webauthn_rs::prelude::Url;
use zeroize::Zeroizing;
mod support;

// Trusted service double only. This is not a membership, network or mTLS test.
#[derive(Clone)]
struct Organization {
    identity: IdentityId,
    mfa: bool,
    denied: bool,
    version: u64,
    absolute: u64,
    step: u64,
}
impl Organization {
    fn new(identity: IdentityId) -> Self {
        Self {
            identity,
            mfa: false,
            denied: false,
            version: 1,
            absolute: 600,
            step: 30,
        }
    }
}
impl OrganizationAuthority for Organization {
    async fn current(
        &mut self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<OrganizationContext, LoginError> {
        if self.denied || identity != self.identity || facility != fid(1) {
            return Err(LoginError);
        }
        OrganizationContext::new(
            identity,
            facility,
            self.version,
            self.absolute,
            self.absolute,
            self.step,
            self.mfa,
        )
    }
}
fn fid(n: u128) -> LoginFacilityId {
    LoginFacilityId::new(n).unwrap()
}
fn sid(n: u128) -> SessionRecordId {
    SessionRecordId::new(n).unwrap()
}
fn iid(n: u128) -> IdentityId {
    IdentityId::new(n).unwrap()
}
fn limits() -> LoginLimits {
    LoginLimits::new(300, 1000, 100).unwrap()
}
fn attempts() -> AttemptLimits {
    AttemptLimits::new(3600, 200, 2000, 100).unwrap()
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .database("kc_step_up_test")
            .username(user)
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn time(c: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}
async fn state(c: &mut PgConnection, n: u128) -> String {
    sqlx::query_scalar("SELECT state FROM identity_access.sessions WHERE id=$1::text::numeric")
        .bind(n.to_string())
        .fetch_one(c)
        .await
        .unwrap()
}
async fn absent(c: &mut PgConnection, n: u128) {
    let missing: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT FROM identity_access.sessions WHERE id=$1::text::numeric)",
    )
    .bind(n.to_string())
    .fetch_one(c)
    .await
    .unwrap();
    assert!(missing);
}
async fn password(
    checker: &PasswordChecker,
    c: &mut PgConnection,
    identity: IdentityId,
    secret: &SessionToken,
    flow: &LoginToken,
) {
    checker
        .check_for_login(
            c,
            AttemptKey::from_server_key(
                identity.value().to_be_bytes().repeat(2).try_into().unwrap(),
            ),
            attempts(),
            Some(identity),
            Password::new(secret.expose()).unwrap(),
            flow,
        )
        .await
        .unwrap();
}
fn code(e: &Enrollment, t: u64) -> TotpCode {
    let totp = totp_rs::Builder::new()
        .with_secret(totp_rs::Secret::new_stack(*e.secret.expose()))
        .build()
        .unwrap();
    let text = Zeroizing::new(totp.generate(t).to_string());
    TotpCode::new(&text).unwrap()
}
async fn passkey(
    service: &LocalhostWebauthn,
    c: &mut PgConnection,
    identity: IdentityId,
    device: &mut SoftPasskey,
    flow: &LoginToken,
) -> bool {
    let (token, options) = service.start_authentication(c, identity).await.unwrap();
    let answer = device
        .perform_auth(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    service
        .finish_authentication_for_login(
            c,
            identity,
            &token,
            &serde_json::to_vec(&answer).unwrap(),
            flow,
        )
        .await
        .is_ok()
}
async fn begin(
    c: &mut PgConnection,
    target: &SessionToken,
    route: LoginRoute,
    org: &mut Organization,
) -> LoginToken {
    login::begin_step_up(c, target, fid(1), route, limits(), org)
        .await
        .unwrap()
}
async fn root(
    c: &mut PgConnection,
    checker: &PasswordChecker,
    secret: &SessionToken,
    org: &mut Organization,
    n: u128,
) -> SessionToken {
    let flow = login::begin(c, org.identity, fid(1), LoginRoute::Password, limits(), org)
        .await
        .unwrap();
    password(checker, c, org.identity, secret, &flow).await;
    login::finish(c, &flow, sid(n), org).await.unwrap()
}
async fn preserved(c: &mut PgConnection, a: u128, b: u128) {
    let same:bool=sqlx::query_scalar("SELECT (p.created_at,p.absolute_expires_at,p.idle_seconds,p.last_activity_at,pb.flow_fingerprint,pb.authenticated_at,pb.mfa) IS NOT DISTINCT FROM (s.created_at,s.absolute_expires_at,s.idle_seconds,s.last_activity_at,sb.flow_fingerprint,sb.authenticated_at,sb.mfa) FROM identity_access.sessions p JOIN identity_access.sessions s ON s.id=$2::text::numeric JOIN identity_access.session_login_bindings pb ON pb.session_id=p.id JOIN identity_access.session_login_bindings sb ON sb.session_id=s.id WHERE p.id=$1::text::numeric")
        .bind(a.to_string()).bind(b.to_string()).fetch_one(c).await.unwrap();
    assert!(same);
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn additional_proof_is_bound_and_rotated() {
    let mut c = connect("postgres").await;
    let cp = pid(&mut c).await;
    let mut witness = connect("postgres").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        assert_eq!(state(&mut c, 1).await, "revoked");
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM identity_access.step_up_receipts")
                .fetch_one(&mut c)
                .await
                .unwrap();
        assert!(count >= 4);
    }
    let base = if after { 10000 } else { 0 };
    let a = iid(base + 1);
    let b = iid(base + 2);
    for identity in [a, b] {
        sqlx::query("INSERT INTO identity_access.identities VALUES($1::text::numeric)")
            .bind(identity.value().to_string())
            .execute(&mut c)
            .await
            .unwrap();
    }
    let checker = PasswordChecker::new().await.unwrap();
    let secret = SessionToken::generate().unwrap();
    for identity in [a, b] {
        checker
            .store(&mut c, identity, Password::new(secret.expose()).unwrap())
            .await
            .unwrap();
    }
    let mut a_org = Organization::new(a);
    a_org.step = 5;
    sqlx::query("INSERT INTO identity_access.mfa_notification_contacts VALUES($1::text::numeric,'00000000-0000-4000-8000-000000000001',0)").bind(a.value().to_string()).execute(&mut c).await.unwrap();
    let mfa = MfaService::new(
        Arc::new(EphemeralMfaKey::generate().unwrap()),
        MfaLimits::new(300, 300, 4, 1, attempts()).unwrap(),
    );
    let enrollment = mfa.begin_enrollment(&mut c, a, None).await.unwrap();
    let mut now = time(&mut c).await;
    // Leave enough space before a TOTP boundary for the previous-step enrollment code.
    if now % 30 > 25 {
        tokio::time::sleep(Duration::from_secs(30 - now % 30)).await;
        now = time(&mut c).await;
    }
    let recovery = mfa
        .activate(
            &mut c,
            a,
            &enrollment,
            &code(&enrollment, now - 30),
            None,
            None,
        )
        .await
        .unwrap();
    let initial = login::begin(
        &mut c,
        a,
        fid(1),
        LoginRoute::Password,
        limits(),
        &mut a_org,
    )
    .await
    .unwrap();
    password(&checker, &mut c, a, &secret, &initial).await;
    mfa.verify_for_login(&mut c, a, &code(&enrollment, now), &initial)
        .await
        .unwrap();
    let old = login::finish(&mut c, &initial, sid(base + 1), &mut a_org)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(6)).await;
    assert!(
        session_check::check(
            &mut c,
            &old,
            fid(1),
            Requirement::RecentMultiFactor,
            &mut a_org
        )
        .await
        .is_err()
    );
    let flow = begin(&mut c, &old, LoginRoute::Password, &mut a_org).await;
    assert!(
        login::finish(&mut c, &flow, sid(base + 1000), &mut a_org)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    password(&checker, &mut c, a, &secret, &flow).await;
    assert!(
        login::finish_step_up(&mut c, &old, &flow, sid(base + 1000), fid(1), &mut a_org)
            .await
            .is_err()
    );
    mfa.verify_for_login(
        &mut c,
        a,
        &code(&enrollment, time(&mut witness).await + 30),
        &flow,
    )
    .await
    .unwrap();
    let fresh = login::finish_step_up(&mut c, &old, &flow, sid(base + 1000), fid(1), &mut a_org)
        .await
        .unwrap();
    assert!(
        session_check::check(
            &mut c,
            &fresh,
            fid(1),
            Requirement::RecentMultiFactor,
            &mut a_org
        )
        .await
        .unwrap()
        .multi_factor()
    );
    assert!(
        session_check::check(&mut c, &old, fid(1), Requirement::Standard, &mut a_org)
            .await
            .is_err()
    );
    preserved(&mut c, base + 1, base + 1000).await;
    assert!(
        login::finish_step_up(&mut c, &fresh, &flow, sid(base + 1999), fid(1), &mut a_org)
            .await
            .is_err()
    );
    absent(&mut c, base + 1999).await;

    // Weak password login may start actual passkey proof after policy requires MFA.
    let webauthn = LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 8).unwrap()).unwrap();
    let mut device = SoftPasskey::new(true);
    let (token, options) = webauthn.start_registration(&mut c, b).await.unwrap();
    let answer = device
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    webauthn
        .finish_registration(&mut c, b, &token, &serde_json::to_vec(&answer).unwrap())
        .await
        .unwrap();
    let mut org = Organization::new(b);
    let weak = root(&mut c, &checker, &secret, &mut org, base + 2).await;
    let sibling = root(&mut c, &checker, &secret, &mut org, base + 4).await;
    org.mfa = true;
    assert!(
        session_check::check(&mut c, &weak, fid(1), Requirement::Standard, &mut org)
            .await
            .is_err()
    );
    assert!(
        login::begin_step_up(
            &mut c,
            &weak,
            fid(2),
            LoginRoute::Passkey,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    let ordinary = login::begin(&mut c, b, fid(1), LoginRoute::Passkey, limits(), &mut org)
        .await
        .unwrap();
    assert!(passkey(&webauthn, &mut c, b, &mut device, &ordinary).await);
    assert!(
        login::finish_step_up(&mut c, &weak, &ordinary, sid(base + 2998), fid(1), &mut org)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    let flow = begin(&mut c, &weak, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    assert!(
        login::finish_step_up(&mut c, &sibling, &flow, sid(base + 2999), fid(1), &mut org)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    absent(&mut c, base + 2999).await;
    assert!(
        login::finish(&mut c, &flow, sid(base + 2000), &mut org)
            .await
            .is_err()
    );
    let strong = login::finish_step_up(&mut c, &weak, &flow, sid(base + 2000), fid(1), &mut org)
        .await
        .unwrap();
    assert!(
        session_check::check(
            &mut c,
            &strong,
            fid(1),
            Requirement::RecentMultiFactor,
            &mut org
        )
        .await
        .unwrap()
        .multi_factor()
    );
    preserved(&mut c, base + 2, base + 2000).await;
    // New evidence survives ordinary rotation, without changing its timestamp.
    let strong = session_rotation::rotate(&mut c, &strong, sid(base + 2001), fid(1), &mut org)
        .await
        .unwrap();
    assert!(
        session_check::check(
            &mut c,
            &strong,
            fid(1),
            Requirement::RecentMultiFactor,
            &mut org
        )
        .await
        .is_ok()
    );

    // Suppressed receipt and deferred commit errors restore proof and original bearer.
    let flow = begin(&mut c, &strong, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    sql(&mut c,"CREATE FUNCTION public.step_suppress() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER step_suppress BEFORE INSERT ON identity_access.step_up_receipts FOR EACH ROW EXECUTE FUNCTION public.step_suppress();").await;
    assert!(
        login::finish_step_up(&mut c, &strong, &flow, sid(base + 2002), fid(1), &mut org)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    absent(&mut c, base + 2002).await;
    assert_eq!(state(&mut c, base + 2001).await, "active");
    sql(&mut c,"DROP TRIGGER step_suppress ON identity_access.step_up_receipts; DROP FUNCTION public.step_suppress(); CREATE FUNCTION public.step_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture commit failure'; END $$; CREATE CONSTRAINT TRIGGER step_fail AFTER INSERT ON identity_access.step_up_receipts DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.step_fail();").await;
    assert!(
        login::finish_step_up(&mut c, &strong, &flow, sid(base + 2002), fid(1), &mut org)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    absent(&mut c, base + 2002).await;
    assert_eq!(state(&mut c, base + 2001).await, "active");
    sql(&mut c,"DROP TRIGGER step_fail ON identity_access.step_up_receipts; DROP FUNCTION public.step_fail();").await;
    let strong = login::finish_step_up(&mut c, &strong, &flow, sid(base + 2002), fid(1), &mut org)
        .await
        .unwrap();

    // Both real contenders wait: one on the target row and one on the identity lock.
    let flow = begin(&mut c, &strong, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    let mut guard = connect("postgres").await;
    let mut lock = guard.begin().await.unwrap();
    sqlx::query("SELECT id FROM identity_access.sessions WHERE id=$1::text::numeric FOR UPDATE")
        .bind((base + 2002).to_string())
        .execute(&mut *lock)
        .await
        .unwrap();
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let pids = [pid(&mut left).await, pid(&mut right).await];
    let mut other_org = org.clone();
    let (l, r, ()) = tokio::join!(
        login::finish_step_up(
            &mut left,
            &strong,
            &flow,
            sid(base + 2003),
            fid(1),
            &mut org
        ),
        login::finish_step_up(
            &mut right,
            &strong,
            &flow,
            sid(base + 2004),
            fid(1),
            &mut other_org
        ),
        async {
            tokio::time::timeout(Duration::from_secs(5),async {loop{let n:i64=sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid=ANY($1) AND wait_event_type='Lock'").bind(pids.as_slice()).fetch_one(&mut witness).await.unwrap();if n==2{break;}tokio::time::sleep(Duration::from_millis(10)).await;}}).await.unwrap();
            lock.commit().await.unwrap();
        }
    );
    assert_eq!(usize::from(l.is_ok()) + usize::from(r.is_ok()), 1);
    support::assert_idle(&mut witness, pids[0]).await;
    support::assert_idle(&mut witness, pids[1]).await;
    let strong = l.or(r).unwrap();
    let changed = begin(&mut c, &strong, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &changed).await);
    let (registration, options) = webauthn.start_registration(&mut c, b).await.unwrap();
    let answer = SoftPasskey::new(true)
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    webauthn
        .finish_registration(
            &mut c,
            b,
            &registration,
            &serde_json::to_vec(&answer).unwrap(),
        )
        .await
        .unwrap();
    assert!(
        login::finish_step_up(
            &mut c,
            &strong,
            &changed,
            sid(base + 2008),
            fid(1),
            &mut org
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, cp).await;
    absent(&mut c, base + 2008).await;
    assert!(
        session_check::check(&mut c, &strong, fid(1), Requirement::Standard, &mut org)
            .await
            .is_ok()
    );
    let flow = begin(&mut c, &strong, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    let rotated = session_rotation::rotate(&mut c, &strong, sid(base + 2005), fid(1), &mut org)
        .await
        .unwrap();
    assert!(
        login::finish_step_up(&mut c, &rotated, &flow, sid(base + 2006), fid(1), &mut org)
            .await
            .is_err()
    );
    assert!(!passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    support::assert_idle(&mut witness, cp).await;
    org.denied = true;
    assert!(
        login::begin_step_up(
            &mut c,
            &rotated,
            fid(1),
            LoginRoute::Passkey,
            limits(),
            &mut org
        )
        .await
        .is_err()
    );
    org.denied = false;
    session_check::logout(&mut c, &weak).await.unwrap();
    assert_eq!(state(&mut c, base + 4).await, "active");
    assert!(
        session_check::check(&mut c, &rotated, fid(1), Requirement::Standard, &mut org)
            .await
            .is_err()
    );

    // An actual receipt INSERT waits in a fixture trigger until the target expires.
    org.version = 2;
    org.mfa = false;
    org.absolute = 3;
    org.step = 3;
    let waiting = root(&mut c, &checker, &secret, &mut org, base + 5).await;
    let pending = begin(&mut c, &waiting, LoginRoute::Passkey, &mut org).await;
    assert!(passkey(&webauthn, &mut c, b, &mut device, &pending).await);
    let deadline: u64 = sqlx::query_scalar::<_, String>(
        "SELECT absolute_expires_at::text FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 5).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap()
    .parse()
    .unwrap();
    sql(&mut c,"CREATE FUNCTION public.step_pause() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(184006,99); RETURN NEW; END $$; CREATE TRIGGER step_pause BEFORE INSERT ON identity_access.step_up_receipts FOR EACH ROW EXECUTE FUNCTION public.step_pause();").await;
    let mut pause = guard.begin().await.unwrap();
    sqlx::query("SELECT pg_advisory_xact_lock(184006,99)")
        .execute(&mut *pause)
        .await
        .unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let (failed, ()) = tokio::join!(
        login::finish_step_up(
            &mut reader,
            &waiting,
            &pending,
            sid(base + 2010),
            fid(1),
            &mut org
        ),
        async {
            tokio::time::timeout(Duration::from_secs(5),async {loop {
                let waiting:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_catalog.pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock')").bind(reader_pid).fetch_one(&mut witness).await.unwrap();
                if waiting{break;}tokio::time::sleep(Duration::from_millis(10)).await;
            }}).await.unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                while time(&mut witness).await < deadline {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .unwrap();
            pause.commit().await.unwrap();
        }
    );
    assert!(failed.is_err());
    support::assert_idle(&mut witness, reader_pid).await;
    assert_eq!(state(&mut c, base + 5).await, "expired");
    absent(&mut c, base + 2010).await;
    sql(&mut c,"DROP TRIGGER step_pause ON identity_access.step_up_receipts; DROP FUNCTION public.step_pause();").await;
    // Expiry after begin is also terminal in the actual factor transaction.
    let short = root(&mut c, &checker, &secret, &mut org, base + 3).await;
    let flow = begin(&mut c, &short, LoginRoute::Passkey, &mut org).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert!(!passkey(&webauthn, &mut c, b, &mut device, &flow).await);
    assert_eq!(state(&mut c, base + 3).await, "expired");
    assert!(
        login::finish_step_up(&mut c, &short, &flow, sid(base + 2007), fid(1), &mut org)
            .await
            .is_err()
    );
    let mut outer = c.begin().await.unwrap();
    assert!(
        login::begin_step_up(
            &mut outer,
            &fresh,
            fid(1),
            LoginRoute::Password,
            limits(),
            &mut a_org
        )
        .await
        .is_err()
    );
    outer.rollback().await.unwrap();
    let mut runtime = connect("kc_step_up_runtime").await;
    assert!(
        login::begin_step_up(
            &mut runtime,
            &fresh,
            fid(1),
            LoginRoute::Password,
            limits(),
            &mut a_org
        )
        .await
        .is_err()
    );
    mfa.recover(&mut c, a, &recovery.codes()[0]).await.unwrap();
    assert!(
        login::begin_step_up(
            &mut c,
            &fresh,
            fid(1),
            LoginRoute::Password,
            limits(),
            &mut a_org
        )
        .await
        .is_err()
    );
}
