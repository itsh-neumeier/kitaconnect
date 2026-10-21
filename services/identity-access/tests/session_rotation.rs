use kitaconnect_identity_access::{
    IdentityId, SessionLimits, SessionRecordId,
    login::{LoginError, LoginFacilityId, OrganizationAuthority, OrganizationContext},
    postgres,
    session_check::{self, Requirement},
    session_rotation,
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
            .database("kc_session_rotation_test")
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
    owner: u128,
    strong: bool,
    age: u64,
    absolute: u64,
) -> SessionToken {
    let at = now(c).await - age;
    let token = SessionToken::generate().unwrap();
    let flow = SessionToken::generate().unwrap();
    let mut tx = c.begin().await.unwrap();
    sqlx::query("INSERT INTO identity_access.identities (id) VALUES ($1::text::numeric) ON CONFLICT DO NOTHING")
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("INSERT INTO identity_access.login_flows (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,mfa_required,created_at,expires_at,last_observed,consumed,proof_digest,password_ok,totp_ok,proven_at) VALUES ($1,$2::text::numeric,10,'password',1,$3::text::numeric,$4::text::numeric,$5::text::numeric,false,$6::text::numeric,$7::text::numeric,$6::text::numeric,true,$8,true,$9,$6::text::numeric)")
        .bind(fingerprint(&flow)).bind(owner.to_string()).bind(900.min(absolute).to_string())
        .bind(absolute.to_string()).bind(300.min(absolute).to_string())
        .bind(at.to_string()).bind((at+300).to_string()).bind(fingerprint(&SessionToken::generate().unwrap()))
        .bind(strong).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO identity_access.sessions (id,identity_id,fingerprint,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,state) VALUES ($1::text::numeric,$6::text::numeric,$2,$3::text::numeric,$4::text::numeric,$5::text::numeric,$3::text::numeric,$3::text::numeric,'active')")
        .bind(id.to_string()).bind(fingerprint(&token)).bind(at.to_string()).bind((at+absolute).to_string())
        .bind(900.min(absolute).to_string()).bind(owner.to_string()).execute(&mut *tx).await.unwrap();
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

fn sid(n: u128) -> SessionRecordId {
    SessionRecordId::new(n).unwrap()
}
async fn absent(c: &mut PgConnection, id: u128) {
    let missing: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT FROM identity_access.sessions WHERE id=$1::text::numeric)",
    )
    .bind(id.to_string())
    .fetch_one(c)
    .await
    .unwrap();
    assert!(missing);
}
async fn unchanged(c: &mut PgConnection, a: u128, b: u128) {
    let same:bool=sqlx::query_scalar("SELECT (a.identity_id,a.created_at,a.absolute_expires_at,a.idle_seconds,a.last_activity_at,ab.flow_fingerprint,ab.authenticated_at,ab.mfa,ao.idle_cap,ao.absolute_cap,ao.step_up_cap,ao.mfa_required) IS NOT DISTINCT FROM (b.identity_id,b.created_at,b.absolute_expires_at,b.idle_seconds,b.last_activity_at,bb.flow_fingerprint,bb.authenticated_at,bb.mfa,bo.idle_cap,bo.absolute_cap,bo.step_up_cap,bo.mfa_required) AND a.fingerprint<>b.fingerprint FROM identity_access.sessions a JOIN identity_access.sessions b ON b.id=$2::text::numeric JOIN identity_access.session_login_bindings ab ON ab.session_id=a.id JOIN identity_access.session_login_bindings bb ON bb.session_id=b.id JOIN identity_access.session_policy_observations ao ON ao.session_id=a.id JOIN identity_access.session_policy_observations bo ON bo.session_id=b.id WHERE a.id=$1::text::numeric")
        .bind(a.to_string()).bind(b.to_string()).fetch_one(c).await.unwrap();
    assert!(same);
}
#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn rotation_preserves_login_and_logout_lineage() {
    if tokio::time::timeout(Duration::from_secs(30), rotation_cases())
        .await
        .is_err()
    {
        let mut diagnostic = connect("postgres").await;
        let rows:Vec<String>=sqlx::query_scalar("SELECT concat(pid,':',state,':',wait_event_type,':',wait_event,':',left(query,120)) FROM pg_stat_activity WHERE datname=current_database() AND pid<>pg_backend_pid() ORDER BY pid")
            .fetch_all(&mut diagnostic).await.unwrap();
        panic!("rotation fixture timeout; database waits: {rows:?}");
    }
}
async fn rotation_cases() {
    println!("rotation fixture: setup and repeated rotation");
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    let base = if after { 10000 } else { 0 };
    let mut c = connect("postgres").await;
    if after {
        for id in [1, 1001, 1002, 4, 1006, 5] {
            assert_ne!(state(&mut c, id).await, "active");
        }
        let links: i64 =
            sqlx::query_scalar("SELECT count(*) FROM identity_access.session_rotation_links")
                .fetch_one(&mut c)
                .await
                .unwrap();
        assert!(links >= 4);
    }
    let first = seed(&mut c, base + 1, base + 1, true, 15, 28800).await;
    let mut a = SyntheticAuthority {
        version: 2,
        idle: 120,
        absolute: 3600,
        step: 60,
        ..Default::default()
    };
    println!("rotation fixture: first rotate");
    let next = session_rotation::rotate(&mut c, &first, sid(base + 1001), facility(), &mut a)
        .await
        .unwrap();
    unchanged(&mut c, base + 1, base + 1001).await;
    assert!(!checked(&mut c, &first, &mut a).await);
    assert!(checked(&mut c, &next, &mut a).await);
    assert!(
        session_rotation::rotate(&mut c, &first, sid(base + 1003), facility(), &mut a)
            .await
            .is_err()
    );
    absent(&mut c, base + 1003).await;
    let last = session_rotation::rotate(&mut c, &next, sid(base + 1002), facility(), &mut a)
        .await
        .unwrap();
    unchanged(&mut c, base + 1001, base + 1002).await;
    assert!(!checked(&mut c, &next, &mut a).await);
    assert!(checked(&mut c, &last, &mut a).await);
    let separate = seed(&mut c, base + 2, base + 1, true, 0, 28800).await;
    assert!(checked(&mut c, &separate, &mut a).await);
    session_check::logout(&mut c, &first).await.unwrap();
    assert!(!checked(&mut c, &last, &mut a).await);
    assert!(
        checked(&mut c, &separate, &mut a).await,
        "logout must not revoke another login of the identity"
    );
    assert!(
        session_rotation::rotate(
            &mut c,
            &separate,
            sid(base + 1003),
            LoginFacilityId::new(11).unwrap(),
            &mut a
        )
        .await
        .is_err()
    );
    a.denied = true;
    assert!(
        session_rotation::rotate(&mut c, &separate, sid(base + 1003), facility(), &mut a)
            .await
            .is_err()
    );
    assert_eq!(state(&mut c, base + 2).await, "active");

    println!("rotation fixture: concurrent rotations");
    // Two rotations are both real lock waiters, never two independent snapshots.
    a = SyntheticAuthority::default();
    let race = seed(&mut c, base + 3, base + 3, true, 0, 28800).await;
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let pids = [pid(&mut left).await, pid(&mut right).await];
    let mut guard_connection = connect("postgres").await;
    let mut guard = guard_connection.begin().await.unwrap();
    sqlx::query("SELECT id FROM identity_access.sessions WHERE id=$1::text::numeric FOR UPDATE")
        .bind((base + 3).to_string())
        .execute(&mut *guard)
        .await
        .unwrap();
    let mut b = a.clone();
    let (x, y, ()) = tokio::join!(
        session_rotation::rotate(&mut left, &race, sid(base + 1004), facility(), &mut a),
        session_rotation::rotate(&mut right, &race, sid(base + 1005), facility(), &mut b),
        async {
            wait(&mut c, pids[0]).await;
            wait(&mut c, pids[1]).await;
            guard.commit().await.unwrap();
        }
    );
    let winning = match (x, y) {
        (Ok(t), Err(_)) | (Err(_), Ok(t)) => t,
        _ => panic!("expected exactly one rotation"),
    };
    assert!(checked(&mut c, &winning, &mut a).await);
    let total:i64=sqlx::query_scalar("SELECT count(*) FROM identity_access.session_rotation_links WHERE predecessor=$1::text::numeric")
        .bind((base+3).to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(total, 1);

    println!("rotation fixture: concurrent logout");
    // Logout waits behind an in-flight rotation, then revokes its newly issued successor.
    let root = seed(&mut c, base + 4, base + 4, true, 0, 28800).await;
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let mut paused = SyntheticAuthority {
        pause: Some((entered.clone(), release.clone())),
        ..Default::default()
    };
    let token = SessionToken::from_bytes(root.expose()).unwrap();
    let mut rotator = connect("postgres").await;
    let rotating = tokio::spawn(async move {
        session_rotation::rotate(
            &mut rotator,
            &token,
            sid(base + 1006),
            facility(),
            &mut paused,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .unwrap();
    let token = SessionToken::from_bytes(root.expose()).unwrap();
    let mut logout_c = connect("postgres").await;
    let logout_pid = pid(&mut logout_c).await;
    let logout = tokio::spawn(async move { session_check::logout(&mut logout_c, &token).await });
    wait(&mut c, logout_pid).await;
    release.notify_one();
    let issued = rotating.await.unwrap().unwrap();
    logout.await.unwrap().unwrap();
    assert!(!checked(&mut c, &issued, &mut a).await);
    assert_eq!(state(&mut c, base + 1006).await, "revoked");

    println!("rotation fixture: suppressed writes and commit failure");
    // Suppressed writes and commit failure must restore the old session.
    let valid = seed(&mut c, base + 6, base + 6, true, 0, 28800).await;
    sql(&mut c,"CREATE FUNCTION public.suppress_rotation() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER suppress_rotation BEFORE INSERT ON identity_access.sessions FOR EACH ROW EXECUTE FUNCTION public.suppress_rotation();").await;
    assert!(
        session_rotation::rotate(&mut c, &valid, sid(base + 1007), facility(), &mut a)
            .await
            .is_err()
    );
    assert_eq!(state(&mut c, base + 6).await, "active");
    assert_eq!(count(&mut c, base + 6).await, 0);
    absent(&mut c, base + 1007).await;
    sql(&mut c,"DROP TRIGGER suppress_rotation ON identity_access.sessions; DROP FUNCTION public.suppress_rotation(); CREATE FUNCTION public.suppress_rotation_link() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER zz_suppress_rotation_link BEFORE INSERT ON identity_access.session_rotation_links FOR EACH ROW EXECUTE FUNCTION public.suppress_rotation_link();").await;
    assert!(
        session_rotation::rotate(&mut c, &valid, sid(base + 1007), facility(), &mut a)
            .await
            .is_err()
    );
    assert_eq!(state(&mut c, base + 6).await, "active");
    absent(&mut c, base + 1007).await;
    sql(&mut c,"DROP TRIGGER zz_suppress_rotation_link ON identity_access.session_rotation_links; DROP FUNCTION public.suppress_rotation_link(); CREATE FUNCTION public.fail_rotation_commit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic commit failure' USING ERRCODE='23514'; END $$; CREATE CONSTRAINT TRIGGER fail_rotation_commit AFTER INSERT ON identity_access.session_rotation_links DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.fail_rotation_commit();").await;
    assert!(
        session_rotation::rotate(&mut c, &valid, sid(base + 1007), facility(), &mut a)
            .await
            .is_err()
    );
    assert_eq!(state(&mut c, base + 6).await, "active");
    absent(&mut c, base + 1007).await;
    sql(&mut c,"DROP TRIGGER fail_rotation_commit ON identity_access.session_rotation_links; DROP FUNCTION public.fail_rotation_commit();").await;
    assert!(checked(&mut c, &valid, &mut a).await);

    println!("rotation fixture: candidate expiry");
    // Candidate insert really waits on a conflicting uncommitted primary key until expiry.
    let expiring = seed(&mut c, base + 5, base + 5, true, 0, 4).await;
    let deadline: u64 = sqlx::query_scalar::<_, String>(
        "SELECT absolute_expires_at::text FROM identity_access.sessions WHERE id=$1::text::numeric",
    )
    .bind((base + 5).to_string())
    .fetch_one(&mut c)
    .await
    .unwrap()
    .parse()
    .unwrap();
    let mut collision_c = connect("postgres").await;
    let mut collision = collision_c.begin().await.unwrap();
    let collision_token = SessionToken::generate().unwrap();
    sqlx::query("INSERT INTO identity_access.sessions (id,identity_id,fingerprint,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,state) SELECT $2::text::numeric,identity_id,$3,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,'active' FROM identity_access.sessions WHERE id=$1::text::numeric")
        .bind((base+5).to_string()).bind((base+1009).to_string()).bind(fingerprint(&collision_token)).execute(&mut *collision).await.unwrap();
    let mut worker = connect("postgres").await;
    let worker_pid = pid(&mut worker).await;
    let token = SessionToken::from_bytes(expiring.expose()).unwrap();
    let mut auth = SyntheticAuthority::default();
    let pending = tokio::spawn(async move {
        session_rotation::rotate(&mut worker, &token, sid(base + 1009), facility(), &mut auth).await
    });
    wait(&mut c, worker_pid).await;
    while now(&mut c).await < deadline {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    collision.rollback().await.unwrap();
    assert!(pending.await.unwrap().is_err());
    assert_eq!(state(&mut c, base + 5).await, "expired");
    absent(&mut c, base + 1009).await;
    let no_link:i64=sqlx::query_scalar("SELECT count(*) FROM identity_access.session_rotation_links WHERE predecessor=$1::text::numeric")
        .bind((base+5).to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(no_link, 0);

    println!("rotation fixture: recovery, outer transaction and RLS");
    // Recovery and an unbound low-level session cannot become a rotation success.
    sqlx::query("INSERT INTO identity_access.mfa_accounts (identity_id,last_observed,recovery_required) VALUES ($1::text::numeric,0,true)")
        .bind((base+6).to_string()).execute(&mut c).await.unwrap();
    assert!(
        session_rotation::rotate(&mut c, &valid, sid(base + 1007), facility(), &mut a)
            .await
            .is_err()
    );
    let raw = postgres::create_session(
        &mut c,
        sid(base + 5000),
        identity(base + 1),
        SessionLimits::new(900, 28800).unwrap(),
    )
    .await
    .unwrap();
    assert!(
        session_rotation::rotate(&mut c, &raw, sid(base + 5001), facility(), &mut a)
            .await
            .is_err()
    );
    let mut outer = c.begin().await.unwrap();
    assert!(
        session_rotation::rotate(&mut outer, &separate, sid(base + 1010), facility(), &mut a)
            .await
            .is_err()
    );
    outer.rollback().await.unwrap();
    let bad=sqlx::query("INSERT INTO identity_access.session_rotation_links (successor,predecessor) VALUES ($1::text::numeric,$2::text::numeric)")
        .bind((base+2).to_string()).bind((base+1).to_string()).execute(&mut c).await.unwrap_err();
    assert_eq!(
        bad.as_database_error().unwrap().code().as_deref(),
        Some("23514")
    );
    let mut runtime = connect("kc_session_rotation_runtime").await;
    let hidden: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.session_rotation_links")
            .fetch_one(&mut runtime)
            .await
            .unwrap();
    assert_eq!(hidden, 0);
    assert!(
        session_rotation::rotate(
            &mut runtime,
            &separate,
            sid(base + 1010),
            facility(),
            &mut a
        )
        .await
        .is_err()
    );
    sql(&mut c, "SET ROLE kc_identity_owner;").await;
    let hidden: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.session_rotation_links")
            .fetch_one(&mut c)
            .await
            .unwrap();
    assert_eq!(hidden, 0);
    sql(&mut c, "RESET ROLE;").await;
}
