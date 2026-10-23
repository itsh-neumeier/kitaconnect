use kitaconnect_identity_access::{
    IdentityId, SessionLimits, SessionRecordId,
    login::{LoginError, LoginFacilityId, OrganizationAuthority, OrganizationContext},
    postgres,
    session_check::{self, Requirement},
    token::SessionToken,
};
use sha2::{Digest, Sha256};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;

// Synthetic service response only: not a real membership store, network peer or mTLS proof.
#[derive(Clone)]
struct SyntheticAuthority {
    version: u64,
    idle: u64,
    absolute: u64,
    step: u64,
    required: bool,
    denied: bool,
    wrong: bool,
    calls: usize,
    delay: Duration,
    pause: Option<(Arc<Notify>, Arc<Notify>)>,
}
impl Default for SyntheticAuthority {
    fn default() -> Self {
        Self {
            version: 1,
            idle: 900,
            absolute: 28800,
            step: 300,
            required: false,
            denied: false,
            wrong: false,
            calls: 0,
            delay: Duration::ZERO,
            pause: None,
        }
    }
}
impl OrganizationAuthority for SyntheticAuthority {
    async fn current(
        &mut self,
        id: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<OrganizationContext, LoginError> {
        self.calls += 1;
        if let Some((entered, release)) = &self.pause {
            entered.notify_one();
            release.notified().await;
        }
        tokio::time::sleep(self.delay).await;
        if self.denied {
            return Err(LoginError);
        }
        OrganizationContext::new(
            if self.wrong {
                identity(id.value() + 1)
            } else {
                id
            },
            facility,
            self.version,
            self.idle,
            self.absolute,
            self.step,
            self.required,
        )
    }
}
fn identity(n: u128) -> IdentityId {
    IdentityId::new(n).unwrap()
}
fn facility() -> LoginFacilityId {
    LoginFacilityId::new(10).unwrap()
}
fn fingerprint(token: &SessionToken) -> Vec<u8> {
    Sha256::digest(token.expose()).to_vec()
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .username(user)
            .database("kc_session_check_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn sql(c: &mut PgConnection, s: &'static str) {
    sqlx::raw_sql(s).execute(c).await.unwrap();
}
async fn now(c: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
async fn state(c: &mut PgConnection, id: u128) -> String {
    sqlx::query_scalar("SELECT state FROM identity_access.sessions WHERE id=$1::text::numeric")
        .bind(id.to_string())
        .fetch_one(c)
        .await
        .unwrap()
}
async fn count(c: &mut PgConnection, id: u128) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM identity_access.session_policy_observations WHERE session_id=$1::text::numeric")
        .bind(id.to_string()).fetch_one(c).await.unwrap()
}
// Explicit administrator fixture for lifecycle tests. Actual factor completion is tested
// separately by login/login_completion_is_atomic_and_current, not implied by this SQL seed.
async fn seed(
    c: &mut PgConnection,
    id: u128,
    strong: bool,
    age: u64,
    absolute: u64,
) -> SessionToken {
    let at = now(c).await - age;
    let token = SessionToken::generate().unwrap();
    let flow = SessionToken::generate().unwrap();
    let mut tx = c.begin().await.unwrap();
    sqlx::query("INSERT INTO identity_access.identities (id) VALUES ($1::text::numeric)")
        .bind(id.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("INSERT INTO identity_access.login_flows (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,mfa_required,created_at,expires_at,last_observed,consumed,proof_digest,password_ok,totp_ok,proven_at) VALUES ($1,$2::text::numeric,10,'password',1,$3::text::numeric,$4::text::numeric,$5::text::numeric,false,$6::text::numeric,$7::text::numeric,$6::text::numeric,true,$8,true,$9,$6::text::numeric)")
        .bind(fingerprint(&flow)).bind(id.to_string()).bind(900.min(absolute).to_string())
        .bind(absolute.to_string()).bind(300.min(absolute).to_string())
        .bind(at.to_string()).bind((at+300).to_string()).bind(fingerprint(&SessionToken::generate().unwrap()))
        .bind(strong).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO identity_access.sessions (id,identity_id,fingerprint,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,state) VALUES ($1::text::numeric,$1::text::numeric,$2,$3::text::numeric,$4::text::numeric,$5::text::numeric,$3::text::numeric,$3::text::numeric,'active')")
        .bind(id.to_string()).bind(fingerprint(&token)).bind(at.to_string()).bind((at+absolute).to_string())
        .bind(900.min(absolute).to_string()).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO identity_access.session_login_bindings (session_id,flow_fingerprint,facility_id,policy_version,step_up_seconds,authenticated_at,mfa) VALUES ($1::text::numeric,$2,10,1,$3::text::numeric,$4::text::numeric,$5)")
        .bind(id.to_string()).bind(fingerprint(&flow)).bind(300.min(absolute).to_string()).bind(at.to_string()).bind(strong)
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    token
}
async fn checked(c: &mut PgConnection, t: &SessionToken, a: &mut SyntheticAuthority) -> bool {
    session_check::check(c, t, facility(), Requirement::Standard, a)
        .await
        .is_ok()
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}
async fn wait(c: &mut PgConnection, pid: i32) {
    tokio::time::timeout(Duration::from_secs(10),async {
        loop {
            let locked:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock')")
                .bind(pid).fetch_one(&mut *c).await.unwrap();
            if locked { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("real identity/row lock waiter required");
}
#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn session_checks_preserve_current_policy_and_logout() {
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    let base = if after { 100 } else { 0 };
    let mut c = connect("postgres").await;
    if after {
        assert_eq!(state(&mut c, 1).await, "revoked");
        assert_eq!(state(&mut c, 4).await, "expired");
        let caps:(i32,i32,i32)=sqlx::query_as("SELECT idle_cap,absolute_cap,step_up_cap FROM identity_access.session_policy_observations WHERE session_id=3")
            .fetch_one(&mut c).await.unwrap();
        assert_eq!(caps, (120, 3600, 60));
    }
    let mut a = SyntheticAuthority::default();
    let good = seed(&mut c, base + 1, true, 10, 28800).await;
    let context = session_check::check(
        &mut c,
        &good,
        facility(),
        Requirement::RecentMultiFactor,
        &mut a,
    )
    .await
    .unwrap();
    assert_eq!(context.session().value(), base + 1);
    assert_eq!(context.identity().value(), base + 1);
    assert_eq!(context.facility(), facility());
    assert!(context.multi_factor());
    let activity: u64 = sqlx::query_scalar::<_, String>(
        "SELECT last_activity_at::text FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 1).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap()
    .parse()
    .unwrap();
    session_check::record_activity(&mut c, &good, facility(), Requirement::Standard, &mut a)
        .await
        .unwrap();
    let later: u64 = sqlx::query_scalar::<_, String>(
        "SELECT last_activity_at::text FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 1).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap()
    .parse()
    .unwrap();
    assert!(later > activity);
    assert!(
        session_check::check(
            &mut c,
            &good,
            LoginFacilityId::new(11).unwrap(),
            Requirement::Standard,
            &mut a
        )
        .await
        .is_err()
    );
    a.denied = true;
    assert!(!checked(&mut c, &good, &mut a).await);
    a.denied = false;
    a.wrong = true;
    assert!(!checked(&mut c, &good, &mut a).await);
    a.wrong = false;
    assert!(checked(&mut c, &good, &mut a).await);
    assert_eq!(
        a.calls, 5,
        "each valid local check must recheck the authority"
    );

    let weak = seed(&mut c, base + 2, false, 0, 28800).await;
    assert!(checked(&mut c, &weak, &mut a).await);
    assert!(
        session_check::check(&mut c, &weak, facility(), Requirement::MultiFactor, &mut a)
            .await
            .is_err()
    );
    a.required = true;
    assert!(!checked(&mut c, &weak, &mut a).await);
    a.required = false;
    assert!(
        !checked(&mut c, &weak, &mut a).await,
        "observed MFA cannot be relaxed for the same session"
    );
    let sticky:bool=sqlx::query_scalar("SELECT mfa_required FROM identity_access.session_policy_observations WHERE session_id=$1::text::numeric")
        .bind((base+2).to_string()).fetch_one(&mut c).await.unwrap();
    assert!(sticky);

    let bounded = seed(&mut c, base + 3, true, 0, 28800).await;
    a.version = 2;
    a.idle = 120;
    a.absolute = 3600;
    a.step = 60;
    assert!(checked(&mut c, &bounded, &mut a).await);
    a = SyntheticAuthority {
        version: 3,
        ..Default::default()
    };
    assert!(checked(&mut c, &bounded, &mut a).await);
    let caps:(i32,i32,i32)=sqlx::query_as("SELECT idle_cap,absolute_cap,step_up_cap FROM identity_access.session_policy_observations WHERE session_id=$1::text::numeric")
        .bind((base+3).to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(caps, (120, 3600, 60));
    for statement in [
        "UPDATE identity_access.session_policy_observations SET idle_cap=121 WHERE session_id=$1::text::numeric",
        "UPDATE identity_access.session_policy_observations SET version=2 WHERE session_id=$1::text::numeric",
        "UPDATE identity_access.session_policy_observations SET raw_idle=800 WHERE session_id=$1::text::numeric",
    ] {
        let error = sqlx::query(sqlx::AssertSqlSafe(statement))
            .bind((base + 3).to_string())
            .execute(&mut c)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23514")
        );
    }
    a.version = 2;
    assert!(!checked(&mut c, &bounded, &mut a).await);
    a.version = 3;
    a.idle = 800;
    assert!(!checked(&mut c, &bounded, &mut a).await);
    a = SyntheticAuthority::default();

    let expired = seed(&mut c, base + 4, true, 20, 28800).await;
    a.idle = 10;
    a.absolute = 1000;
    a.step = 10;
    assert!(!checked(&mut c, &expired, &mut a).await);
    assert_eq!(state(&mut c, base + 4).await, "expired");
    a = SyntheticAuthority {
        version: 2,
        ..Default::default()
    };
    assert!(!checked(&mut c, &expired, &mut a).await);

    let stale = seed(&mut c, base + 5, true, 20, 28800).await;
    a = SyntheticAuthority {
        step: 10,
        ..Default::default()
    };
    assert!(
        session_check::check(
            &mut c,
            &stale,
            facility(),
            Requirement::RecentMultiFactor,
            &mut a
        )
        .await
        .is_err()
    );
    assert!(checked(&mut c, &stale, &mut a).await);
    let lost = seed(&mut c, base + 6, true, 0, 2).await;
    a = SyntheticAuthority {
        delay: Duration::from_secs(3),
        ..Default::default()
    };
    assert!(!checked(&mut c, &lost, &mut a).await);
    assert_eq!(state(&mut c, base + 6).await, "expired");

    // Previously shortened limits expire locally even if the current authority fails.
    let old_caps = seed(&mut c, base + 10, true, 0, 28800).await;
    a = SyntheticAuthority {
        idle: 2,
        absolute: 1000,
        step: 1,
        ..Default::default()
    };
    assert!(checked(&mut c, &old_caps, &mut a).await);
    tokio::time::sleep(Duration::from_secs(3)).await;
    a.denied = true;
    let calls = a.calls;
    assert!(!checked(&mut c, &old_caps, &mut a).await);
    assert_eq!(state(&mut c, base + 10).await, "expired");
    assert_eq!(
        a.calls, calls,
        "stored expiry must be handled before an unavailable authority"
    );
    a = SyntheticAuthority {
        version: 2,
        ..Default::default()
    };
    assert!(!checked(&mut c, &old_caps, &mut a).await);

    // At issuance, original caps may already differ from the current raw policy.
    let already_short = seed(&mut c, base + 11, true, 0, 600).await;
    sqlx::query("UPDATE identity_access.session_login_bindings SET policy_version=2 WHERE session_id=$1::text::numeric")
        .bind((base+11).to_string()).execute(&mut c).await.unwrap();
    assert!(checked(&mut c, &already_short, &mut a).await);
    a = SyntheticAuthority::default();

    let recovery = seed(&mut c, base + 7, true, 0, 28800).await;
    sqlx::query("INSERT INTO identity_access.mfa_accounts (identity_id,last_observed,recovery_required) VALUES ($1::text::numeric,0,true)")
        .bind((base+7).to_string()).execute(&mut c).await.unwrap();
    assert!(!checked(&mut c, &recovery, &mut a).await);
    session_check::logout(&mut c, &recovery).await.unwrap();
    assert_eq!(state(&mut c, base + 7).await, "revoked");

    let raw = postgres::create_session(
        &mut c,
        SessionRecordId::new(base + 500).unwrap(),
        identity(base + 1),
        SessionLimits::new(900, 28800).unwrap(),
    )
    .await
    .unwrap();
    assert!(!checked(&mut c, &raw, &mut a).await);
    let unknown = SessionToken::generate().unwrap();
    assert!(!checked(&mut c, &unknown, &mut a).await);
    session_check::logout(&mut c, &unknown).await.unwrap();

    let failing = seed(&mut c, base + 8, true, 10, 28800).await;
    sql(&mut c,"CREATE FUNCTION public.suppress_observation() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER suppress_observation BEFORE INSERT ON identity_access.session_policy_observations FOR EACH ROW EXECUTE FUNCTION public.suppress_observation();").await;
    assert!(!checked(&mut c, &failing, &mut a).await);
    assert_eq!(count(&mut c, base + 8).await, 0);
    sql(&mut c,"DROP TRIGGER suppress_observation ON identity_access.session_policy_observations; DROP FUNCTION public.suppress_observation(); CREATE FUNCTION public.fail_observation() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic commit failure' USING ERRCODE='23514'; END $$; CREATE CONSTRAINT TRIGGER fail_observation AFTER INSERT ON identity_access.session_policy_observations DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.fail_observation();").await;
    assert!(
        session_check::record_activity(&mut c, &failing, facility(), Requirement::Standard, &mut a)
            .await
            .is_err()
    );
    assert_eq!(count(&mut c, base + 8).await, 0);
    let unchanged:bool=sqlx::query_scalar("SELECT last_activity_at=created_at FROM identity_access.sessions WHERE id=$1::text::numeric")
        .bind((base+8).to_string()).fetch_one(&mut c).await.unwrap();
    assert!(unchanged);
    sql(&mut c,"DROP TRIGGER fail_observation ON identity_access.session_policy_observations; DROP FUNCTION public.fail_observation();").await;
    assert!(checked(&mut c, &failing, &mut a).await);
    sql(&mut c,"CREATE FUNCTION public.suppress_session() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER suppress_session BEFORE UPDATE ON identity_access.sessions FOR EACH ROW EXECUTE FUNCTION public.suppress_session();").await;
    assert!(session_check::logout(&mut c, &failing).await.is_err());
    assert_eq!(state(&mut c, base + 8).await, "active");
    sql(&mut c,"DROP TRIGGER suppress_session ON identity_access.sessions; DROP FUNCTION public.suppress_session();").await;

    let regressed = seed(&mut c, base + 9, true, 0, 28800).await;
    let at = now(&mut c).await;
    sqlx::query("UPDATE identity_access.sessions SET last_observed_at=$2::text::numeric WHERE id=$1::text::numeric")
        .bind((base+9).to_string()).bind((at+30).to_string()).execute(&mut c).await.unwrap();
    assert!(!checked(&mut c, &regressed, &mut a).await);
    let mut outer = c.begin().await.unwrap();
    assert!(
        session_check::check(&mut outer, &good, facility(), Requirement::Standard, &mut a)
            .await
            .is_err()
    );
    assert!(session_check::logout(&mut outer, &good).await.is_err());
    outer.rollback().await.unwrap();

    // Hold an actual session check inside its organization call; logout must wait.
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let mut paused = SyntheticAuthority {
        pause: Some((entered.clone(), release.clone())),
        ..Default::default()
    };
    let token = SessionToken::from_bytes(good.expose()).unwrap();
    let mut left = connect("postgres").await;
    let task = tokio::spawn(async move {
        session_check::check(
            &mut left,
            &token,
            facility(),
            Requirement::Standard,
            &mut paused,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .unwrap();
    let mut right = connect("postgres").await;
    let logout_pid = pid(&mut right).await;
    let token = SessionToken::from_bytes(good.expose()).unwrap();
    let logout = tokio::spawn(async move { session_check::logout(&mut right, &token).await });
    wait(&mut c, logout_pid).await;
    release.notify_one();
    task.await.unwrap().unwrap();
    logout.await.unwrap().unwrap();
    assert!(!checked(&mut c, &good, &mut a).await);
    session_check::logout(&mut c, &good).await.unwrap();
    assert_eq!(state(&mut c, base + 1).await, "revoked");

    let mut runtime = connect("kc_session_check_runtime").await;
    let visible: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.session_policy_observations")
            .fetch_one(&mut runtime)
            .await
            .unwrap();
    assert_eq!(visible, 0);
    assert!(!checked(&mut runtime, &bounded, &mut a).await);
    sql(&mut c, "SET ROLE kc_identity_owner;").await;
    let owner: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.session_policy_observations")
            .fetch_one(&mut c)
            .await
            .unwrap();
    assert_eq!(owner, 0);
    sql(&mut c, "RESET ROLE;").await;
    let protected:bool=sqlx::query_scalar("SELECT relrowsecurity AND relforcerowsecurity FROM pg_class WHERE oid='identity_access.session_policy_observations'::regclass")
        .fetch_one(&mut c).await.unwrap();
    assert!(protected);
    let policies:i64=sqlx::query_scalar("SELECT count(*) FROM pg_policies WHERE schemaname='identity_access' AND tablename='session_policy_observations'")
        .fetch_one(&mut c).await.unwrap();
    assert_eq!(policies, 0);
    assert!(format!("{:?}", session_check::CheckError).contains("unavailable"));
}
