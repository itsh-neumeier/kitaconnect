use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use kitaconnect_identity_access::{
    IdentityId, SessionLimits, SessionRecordId,
    password::{AttemptKey, AttemptLimits, Password, PasswordChecker, PasswordError},
    postgres,
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
            .database("kc_password_test")
            .username(user)
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
fn id() -> IdentityId {
    IdentityId::new(1).unwrap()
}
fn input() -> Password {
    Password::new(b"synthetic password input for isolated tests").unwrap()
}
fn key(value: u8) -> AttemptKey {
    AttemptKey::from_server_key([value; 32])
}
fn limits() -> AttemptLimits {
    AttemptLimits::new(86_400, 30, 100, 20).unwrap()
}
async fn reset(connection: &mut PgConnection) {
    sqlx::raw_sql("DELETE FROM identity_access.password_attempts; DELETE FROM identity_access.password_budget;").execute(connection).await.unwrap();
}
async fn used(connection: &mut PgConnection) -> i64 {
    sqlx::query_scalar("SELECT used FROM identity_access.password_budget WHERE id = 1")
        .fetch_one(connection)
        .await
        .unwrap()
}
async fn rejected(
    checker: &PasswordChecker,
    connection: &mut PgConnection,
    identity: Option<IdentityId>,
    password: Password,
) {
    let error = checker
        .check(connection, key(1), limits(), identity, password)
        .await
        .unwrap_err();
    assert_eq!(error, PasswordError);
    assert_eq!(
        format!("{error:?}"),
        "password check unavailable or unsuccessful"
    );
}
async fn waiters(connection: &mut PgConnection, pids: &[i32]) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid = ANY($1) AND wait_event_type = 'Lock' AND state = 'active'")
                .bind(pids).fetch_one(&mut *connection).await.unwrap();
            if count == pids.len() as i64 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("real PostgreSQL lock wait required");
}
async fn pid(connection: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(connection)
        .await
        .unwrap()
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn passwords_and_durable_attempts_fail_closed() {
    let mut connection = connect("postgres").await;
    let phase = std::env::var("KC_ORG_TEST_PHASE").unwrap();
    let checker = std::sync::Arc::new(PasswordChecker::new().await.unwrap());
    if phase == "after" {
        assert_eq!(used(&mut connection).await, 1);
        let error = checker
            .check(
                &mut connection,
                key(42),
                AttemptLimits::new(86_400, 1, 1, 1).unwrap(),
                Some(id()),
                input(),
            )
            .await
            .unwrap_err();
        assert_eq!(error, PasswordError);
        assert_eq!(used(&mut connection).await, 1);
        let versions: Vec<i32> = sqlx::query_scalar(
            "SELECT version FROM identity_access.schema_versions ORDER BY version",
        )
        .fetch_all(&mut connection)
        .await
        .unwrap();
        assert_eq!(versions, [1, 2]);
    }
    reset(&mut connection).await;
    sqlx::query("INSERT INTO identity_access.identities VALUES (1) ON CONFLICT DO NOTHING")
        .execute(&mut connection)
        .await
        .unwrap();
    checker.store(&mut connection, id(), input()).await.unwrap();
    let sessions_before: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.sessions")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    let evidence = checker
        .check(&mut connection, key(1), limits(), Some(id()), input())
        .await
        .unwrap();
    assert_eq!(evidence.identity(), id());
    assert_eq!(format!("{evidence:?}"), "PasswordEvidence([REDACTED])");
    rejected(
        &checker,
        &mut connection,
        Some(id()),
        Password::new(b"wrong synthetic password").unwrap(),
    )
    .await;
    rejected(&checker, &mut connection, None, input()).await;
    rejected(
        &checker,
        &mut connection,
        Some(IdentityId::new(999).unwrap()),
        input(),
    )
    .await;
    assert_eq!(used(&mut connection).await, 4);
    let sessions_after: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.sessions")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(sessions_before, sessions_after);

    // Check actual full-size parameter upgrade with a fresh OS-random salt.
    let legacy = tokio::task::spawn_blocking(|| {
        let mut salt = [0; 16];
        getrandom::fill(&mut salt).unwrap();
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19_456, 2, 1, Some(32)).unwrap(),
        )
        .hash_password_with_salt(b"synthetic password input for isolated tests", &salt)
        .unwrap()
        .to_string()
    })
    .await
    .unwrap();
    sqlx::query("UPDATE identity_access.password_credentials SET phc = $1 WHERE identity_id = 1")
        .bind(&legacy)
        .execute(&mut connection)
        .await
        .unwrap();
    rejected(
        &checker,
        &mut connection,
        Some(id()),
        Password::new(b"wrong synthetic input").unwrap(),
    )
    .await;
    let unchanged: bool = sqlx::query_scalar(
        "SELECT phc = $1 FROM identity_access.password_credentials WHERE identity_id = 1",
    )
    .bind(&legacy)
    .fetch_one(&mut connection)
    .await
    .unwrap();
    assert!(unchanged);
    checker
        .check(&mut connection, key(1), limits(), Some(id()), input())
        .await
        .unwrap();
    let upgraded: bool = sqlx::query_scalar("SELECT phc LIKE '$argon2id$v=19$m=65536,t=3,p=1$%' FROM identity_access.password_credentials WHERE identity_id = 1")
        .fetch_one(&mut connection).await.unwrap();
    assert!(upgraded);
    // Bogus format and excessive parameters fail with the same public error.
    for invalid in [
        "malformed",
        "$argon2id$v=19$m=4294967295,t=3,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ] {
        sqlx::query(
            "UPDATE identity_access.password_credentials SET phc = $1 WHERE identity_id = 1",
        )
        .bind(invalid)
        .execute(&mut connection)
        .await
        .unwrap();
        rejected(&checker, &mut connection, Some(id()), input()).await;
    }
    checker.store(&mut connection, id(), input()).await.unwrap();
    let token = postgres::create_session(
        &mut connection,
        SessionRecordId::new(if phase == "before" { 1 } else { 2 }).unwrap(),
        id(),
        SessionLimits::new(1_000, 10_000).unwrap(),
    )
    .await
    .unwrap();
    checker.store(&mut connection, id(), input()).await.unwrap();
    assert!(
        postgres::inspect(&mut connection, &token)
            .await
            .unwrap()
            .is_none()
    );

    // A positive password snapshot is rejected if the stored revision changes while final validation waits.
    let mut guard = connect("postgres").await;
    let mut guarded = guard.begin().await.unwrap();
    sqlx::query("SELECT identity_id FROM identity_access.password_credentials WHERE identity_id = 1 FOR UPDATE").execute(&mut *guarded).await.unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let worker = std::sync::Arc::clone(&checker);
    let pending = tokio::spawn(async move {
        worker
            .check(&mut reader, key(1), limits(), Some(id()), input())
            .await
    });
    waiters(&mut connection, &[reader_pid]).await;
    let mut revision = [0; 32];
    getrandom::fill(&mut revision).unwrap();
    sqlx::query(
        "UPDATE identity_access.password_credentials SET revision = $1 WHERE identity_id = 1",
    )
    .bind(revision.as_slice())
    .execute(&mut *guarded)
    .await
    .unwrap();
    guarded.commit().await.unwrap();
    assert_eq!(pending.await.unwrap().unwrap_err(), PasswordError);

    // Also reject a changed PHC if maintenance did not change its revision.
    let mut guarded = guard.begin().await.unwrap();
    sqlx::query("SELECT identity_id FROM identity_access.password_credentials WHERE identity_id = 1 FOR UPDATE").execute(&mut *guarded).await.unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let worker = std::sync::Arc::clone(&checker);
    let pending = tokio::spawn(async move {
        worker
            .check(&mut reader, key(1), limits(), Some(id()), input())
            .await
    });
    waiters(&mut connection, &[reader_pid]).await;
    sqlx::query(
        "UPDATE identity_access.password_credentials SET phc = 'invalidated' WHERE identity_id = 1",
    )
    .execute(&mut *guarded)
    .await
    .unwrap();
    guarded.commit().await.unwrap();
    assert_eq!(pending.await.unwrap().unwrap_err(), PasswordError);
    checker.store(&mut connection, id(), input()).await.unwrap();

    // Open caller transactions cannot roll back an already accepted attempt reservation.
    let mut transaction = connection.begin().await.unwrap();
    assert_eq!(
        checker
            .check(&mut transaction, key(1), limits(), Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(
        checker
            .store(&mut transaction, id(), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    transaction.rollback().await.unwrap();

    // Runtime stays blocked for credentials, counters, maintenance and positive factor results.
    let mut runtime = connect("kc_identity_test_runtime").await;
    assert_eq!(
        checker
            .check(&mut runtime, key(1), limits(), Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(
        checker
            .store(&mut runtime, id(), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    for query in [
        "SELECT count(*) FROM identity_access.password_credentials",
        "SELECT count(*) FROM identity_access.password_budget",
        "SELECT count(*) FROM identity_access.password_attempts",
    ] {
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&mut runtime)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    sqlx::query("SET ROLE kc_identity_owner")
        .execute(&mut runtime)
        .await
        .unwrap_err();
    let mut owner = connect("postgres").await;
    sqlx::query("SET ROLE kc_identity_owner")
        .execute(&mut owner)
        .await
        .unwrap();
    for query in [
        "SELECT count(*) FROM identity_access.password_credentials",
        "SELECT count(*) FROM identity_access.password_budget",
        "SELECT count(*) FROM identity_access.password_attempts",
    ] {
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&mut owner)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    // Two real waiters compete for one remaining reservation. Both negative checks consume at most one attempt.
    reset(&mut connection).await;
    let tight = AttemptLimits::new(86_400, 1, 1, 1).unwrap();
    sqlx::query("INSERT INTO identity_access.password_budget VALUES (1, 86400, 1, 1, 1, floor(EXTRACT(epoch FROM clock_timestamp())), floor(EXTRACT(epoch FROM clock_timestamp())), 0)").execute(&mut connection).await.unwrap();
    let mut guarded = guard.begin().await.unwrap();
    sqlx::query("SELECT id FROM identity_access.password_budget WHERE id = 1 FOR UPDATE")
        .execute(&mut *guarded)
        .await
        .unwrap();
    let mut a = connect("postgres").await;
    let mut b = connect("postgres").await;
    let pids = [pid(&mut a).await, pid(&mut b).await];
    let worker = std::sync::Arc::clone(&checker);
    let a = tokio::spawn(async move { worker.check(&mut a, key(42), tight, None, input()).await });
    let worker = std::sync::Arc::clone(&checker);
    let b = tokio::spawn(async move { worker.check(&mut b, key(42), tight, None, input()).await });
    waiters(&mut connection, &pids).await;
    guarded.commit().await.unwrap();
    assert_eq!(a.await.unwrap().unwrap_err(), PasswordError);
    assert_eq!(b.await.unwrap().unwrap_err(), PasswordError);
    assert_eq!(used(&mut connection).await, 1);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.password_attempts")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Per-key, global and active-key limits are independent, do not reset on success, and reject changed policy.
    reset(&mut connection).await;
    let bounded = AttemptLimits::new(86_400, 1, 3, 2).unwrap();
    checker
        .check(&mut connection, key(10), bounded, Some(id()), input())
        .await
        .unwrap();
    assert_eq!(
        checker
            .check(&mut connection, key(10), bounded, Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(used(&mut connection).await, 1);
    checker
        .check(&mut connection, key(11), bounded, Some(id()), input())
        .await
        .unwrap();
    assert_eq!(
        checker
            .check(&mut connection, key(12), bounded, Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(used(&mut connection).await, 2);
    assert_eq!(
        checker
            .check(&mut connection, key(11), limits(), Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(used(&mut connection).await, 2);
    sqlx::query("UPDATE identity_access.password_budget SET last_observed = floor(EXTRACT(epoch FROM clock_timestamp())) + 3600").execute(&mut connection).await.unwrap();
    assert_eq!(
        checker
            .check(&mut connection, key(12), bounded, Some(id()), input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(used(&mut connection).await, 2);
    sqlx::raw_sql("UPDATE identity_access.password_budget SET window_start = 0, last_observed = 0; UPDATE identity_access.password_attempts SET window_start = 0;").execute(&mut connection).await.unwrap();
    checker
        .check(&mut connection, key(12), bounded, Some(id()), input())
        .await
        .unwrap();
    assert_eq!(used(&mut connection).await, 1);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.password_attempts")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Leave a real exhausted reservation, without raw password or session material, across server restart.
    reset(&mut connection).await;
    assert_eq!(
        checker
            .check(&mut connection, key(42), tight, None, input())
            .await
            .unwrap_err(),
        PasswordError
    );
    assert_eq!(used(&mut connection).await, 1);
}
