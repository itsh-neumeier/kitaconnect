use kitaconnect_identity_access::{
    IdentityId, SessionError, SessionLimits, SessionRecordId,
    postgres::{self, StorageError},
    token::SessionToken,
};
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{Connection, PgConnection};
use std::time::Duration;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated test socket required"))
            .username(user)
            .database("kc_identity_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
fn id(value: u128) -> SessionRecordId {
    SessionRecordId::new(value).unwrap()
}
fn identity(value: u128) -> IdentityId {
    IdentityId::new(value).unwrap()
}
fn limits() -> SessionLimits {
    SessionLimits::new(1_000, 10_000).unwrap()
}

async fn execute(connection: &mut PgConnection, sql: &'static str) {
    sqlx::raw_sql(sql).execute(connection).await.unwrap();
}
fn database_code(error: StorageError, expected: &str) {
    assert_eq!(format!("{error:?}"), "session storage operation failed");
    match error {
        StorageError::Database(error) => assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some(expected)
        ),
        _ => panic!("expected database failure"),
    }
}
async fn rejected(connection: &mut PgConnection, sql: &'static str, code: &str) {
    let error = sqlx::query(sql).execute(connection).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some(code)
    );
}
async fn seconds(connection: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(connection)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
async fn state(connection: &mut PgConnection, key: u128) -> String {
    sqlx::query_scalar("SELECT state FROM identity_access.sessions WHERE id = $1::text::numeric")
        .bind(key.to_string())
        .fetch_one(connection)
        .await
        .unwrap()
}
async fn pid(connection: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(connection)
        .await
        .unwrap()
}
async fn waiters(witness: &mut PgConnection, expected_pids: &[i32]) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid = ANY($1) AND wait_event_type = 'Lock' AND state = 'active'")
                .bind(expected_pids).fetch_one(&mut *witness).await.unwrap();
            if waiting == expected_pids.len() as i64 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("expected real row-lock waiters");
}

struct Times {
    created: u64,
    activity: u64,
    observed: u64,
    absolute: u64,
    idle: u64,
}
async fn seed(connection: &mut PgConnection, key: u128, token: &SessionToken, times: Times) {
    sqlx::query("INSERT INTO identity_access.sessions (id, identity_id, fingerprint, created_at, last_activity_at, last_observed_at, absolute_expires_at, idle_seconds, state) VALUES ($1::text::numeric, 1, $2, $3::text::numeric, $4::text::numeric, $5::text::numeric, $6::text::numeric, $7::text::numeric, 'active')")
        .bind(key.to_string()).bind(Sha256::digest(token.expose()).as_slice())
        .bind(times.created.to_string()).bind(times.activity.to_string()).bind(times.observed.to_string())
        .bind(times.absolute.to_string()).bind(times.idle.to_string()).execute(connection).await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn current_sessions_are_atomic_and_fail_closed() {
    let phase = std::env::var("KC_ORG_TEST_PHASE").unwrap();
    assert!(matches!(phase.as_str(), "before" | "after"));
    let base = if phase == "before" { 1_000 } else { 2_000 };
    let mut admin = connect("postgres").await;
    if phase == "before" {
        postgres::register_identity(&mut admin, identity(1))
            .await
            .unwrap();
        postgres::register_identity(&mut admin, identity(u128::MAX))
            .await
            .unwrap();
        let token =
            postgres::create_session(&mut admin, id(u128::MAX), identity(u128::MAX), limits())
                .await
                .unwrap();
        let snapshot = postgres::inspect(&mut admin, &token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.id(), id(u128::MAX));
        assert_eq!(snapshot.identity(), identity(u128::MAX));
        postgres::revoke(&mut admin, id(u128::MAX)).await.unwrap();
    } else {
        for (key, expected) in [
            (1001, "revoked"),
            (1002, "active"),
            (1003, "expired"),
            (1004, "expired"),
            (1005, "revoked"),
            (1006, "revoked"),
            (1007, "expired"),
            (u128::MAX, "revoked"),
        ] {
            assert_eq!(
                state(&mut admin, key).await,
                expected,
                "persisted lifecycle state"
            );
        }
    }
    // Suppressed INSERT commands must not report a registered identity or return a token.
    execute(&mut admin, "CREATE FUNCTION identity_access.test_suppress_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER z_test_suppress_insert BEFORE INSERT ON identity_access.identities FOR EACH ROW EXECUTE FUNCTION identity_access.test_suppress_insert(); CREATE TRIGGER z_test_suppress_insert BEFORE INSERT ON identity_access.sessions FOR EACH ROW EXECUTE FUNCTION identity_access.test_suppress_insert();").await;
    assert!(matches!(
        postgres::register_identity(&mut admin, identity(base + 88)).await,
        Err(StorageError::WriteNotConfirmed)
    ));
    assert!(matches!(
        postgres::create_session(&mut admin, id(base + 88), identity(1), limits()).await,
        Err(StorageError::WriteNotConfirmed)
    ));
    let absent: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT FROM identity_access.identities WHERE id=$1::text::numeric) AND NOT EXISTS(SELECT FROM identity_access.sessions WHERE id=$1::text::numeric)")
        .bind((base + 88).to_string()).fetch_one(&mut admin).await.unwrap();
    assert!(absent);
    execute(&mut admin, "DROP TRIGGER z_test_suppress_insert ON identity_access.identities; DROP TRIGGER z_test_suppress_insert ON identity_access.sessions; DROP FUNCTION identity_access.test_suppress_insert();").await;
    let version: Vec<i32> =
        sqlx::query_scalar("SELECT version FROM identity_access.schema_versions ORDER BY version")
            .fetch_all(&mut admin)
            .await
            .unwrap();
    assert_eq!(version, [1]);
    let protected: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'identity_access' AND c.relname IN ('identities', 'sessions') AND c.relrowsecurity AND c.relforcerowsecurity AND pg_catalog.pg_get_userbyid(c.relowner) = 'kc_identity_owner'")
        .fetch_one(&mut admin).await.unwrap();
    assert_eq!(protected, 2);
    let policies: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_catalog.pg_policies WHERE schemaname = 'identity_access'",
    )
    .fetch_one(&mut admin)
    .await
    .unwrap();
    assert_eq!(policies, 0);
    for sql in [
        "SELECT 0::identity_access.internal_id",
        "SELECT 1.5::identity_access.internal_id",
        "SELECT 340282366920938463463374607431768211456::identity_access.internal_id",
        "SELECT (-1)::identity_access.epoch_second",
        "SELECT 1.5::identity_access.epoch_second",
        "SELECT 18446744073709551616::identity_access.epoch_second",
        "SELECT 0::identity_access.positive_seconds",
    ] {
        rejected(&mut admin, sql, "23514").await;
    }
    database_code(
        postgres::register_identity(&mut admin, identity(1))
            .await
            .unwrap_err(),
        "23505",
    );
    database_code(
        postgres::create_session(&mut admin, id(base + 99), identity(999), limits())
            .await
            .unwrap_err(),
        "23503",
    );

    let token = postgres::create_session(&mut admin, id(base + 1), identity(1), limits())
        .await
        .unwrap();
    let stored: Vec<u8> = sqlx::query_scalar(
        "SELECT fingerprint FROM identity_access.sessions WHERE id = $1::text::numeric",
    )
    .bind((base + 1).to_string())
    .fetch_one(&mut admin)
    .await
    .unwrap();
    assert!(
        stored.as_slice() == Sha256::digest(token.expose()).as_slice(),
        "wrong fingerprint"
    );
    assert!(
        stored.as_slice() != token.expose(),
        "raw bearer material stored"
    );
    database_code(
        postgres::create_session(&mut admin, id(base + 1), identity(1), limits())
            .await
            .unwrap_err(),
        "23505",
    );
    let unknown = SessionToken::generate().unwrap();
    assert!(
        postgres::inspect(&mut admin, &unknown)
            .await
            .unwrap()
            .is_none()
    );
    let initial = postgres::inspect(&mut admin, &token)
        .await
        .unwrap()
        .unwrap();
    let observed = postgres::inspect(&mut admin, &token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(observed.last_activity_at(), initial.last_activity_at());
    assert_eq!(
        observed.absolute_expires_at(),
        initial.absolute_expires_at()
    );
    assert!(postgres::revoke(&mut admin, id(base + 1)).await.unwrap());
    assert!(postgres::revoke(&mut admin, id(base + 1)).await.unwrap());
    assert!(
        postgres::inspect(&mut admin, &token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        postgres::record_activity(&mut admin, &token)
            .await
            .unwrap()
            .is_none()
    );
    rejected(
        &mut admin,
        "UPDATE identity_access.sessions SET state = 'active' WHERE state = 'revoked'",
        "23514",
    )
    .await;
    rejected(
        &mut admin,
        "UPDATE identity_access.sessions SET absolute_expires_at = absolute_expires_at + 1",
        "23514",
    )
    .await;

    let activity = postgres::create_session(&mut admin, id(base + 2), identity(1), limits())
        .await
        .unwrap();
    let first = postgres::inspect(&mut admin, &activity)
        .await
        .unwrap()
        .unwrap();
    let touched = postgres::record_activity(&mut admin, &activity)
        .await
        .unwrap()
        .unwrap();
    assert!(touched.last_activity_at() >= first.last_activity_at());
    assert_eq!(touched.absolute_expires_at(), first.absolute_expires_at());
    let mut outer = admin.begin().await.unwrap();
    assert!(matches!(
        postgres::inspect(&mut outer, &activity).await,
        Err(StorageError::TransactionActive)
    ));
    assert!(matches!(
        postgres::revoke(&mut outer, id(base + 2)).await,
        Err(StorageError::TransactionActive)
    ));
    assert!(matches!(
        postgres::create_session(&mut outer, id(base + 80), identity(1), limits()).await,
        Err(StorageError::TransactionActive)
    ));
    assert!(matches!(
        postgres::register_identity(&mut outer, identity(80)).await,
        Err(StorageError::TransactionActive)
    ));
    outer.rollback().await.unwrap();
    assert!(matches!(
        postgres::create_session(
            &mut admin,
            id(base + 81),
            identity(1),
            SessionLimits::new(1, u64::MAX).unwrap()
        )
        .await,
        Err(StorageError::Model(SessionError::TimeOverflow))
    ));

    let time = seconds(&mut admin).await;
    for (key, times) in [
        (
            base + 3,
            Times {
                created: time - 20,
                activity: time - 10,
                observed: time - 10,
                absolute: time + 100,
                idle: 5,
            },
        ),
        (
            base + 4,
            Times {
                created: time - 20,
                activity: time - 15,
                observed: time - 15,
                absolute: time - 5,
                idle: 100,
            },
        ),
    ] {
        let expired = SessionToken::generate().unwrap();
        seed(&mut admin, key, &expired, times).await;
        assert!(
            postgres::inspect(&mut admin, &expired)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(state(&mut admin, key).await, "expired");
        assert!(
            postgres::record_activity(&mut admin, &expired)
                .await
                .unwrap()
                .is_none()
        );
    }
    let future = SessionToken::generate().unwrap();
    seed(
        &mut admin,
        base + 5,
        &future,
        Times {
            created: time,
            activity: time,
            observed: time + 100,
            absolute: time + 1000,
            idle: 2000,
        },
    )
    .await;
    assert!(matches!(
        postgres::inspect(&mut admin, &future).await,
        Err(StorageError::Model(SessionError::ClockRegression))
    ));
    postgres::revoke(&mut admin, id(base + 5)).await.unwrap();

    // An unconfirmed database update cannot produce a positive session snapshot.
    execute(&mut admin, "CREATE FUNCTION identity_access.test_suppress_update() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER z_test_suppress BEFORE UPDATE ON identity_access.sessions FOR EACH ROW EXECUTE FUNCTION identity_access.test_suppress_update()").await;
    let suppressed = postgres::record_activity(&mut admin, &activity).await;
    execute(&mut admin, "DROP TRIGGER z_test_suppress ON identity_access.sessions; DROP FUNCTION identity_access.test_suppress_update()").await;
    assert!(matches!(suppressed, Err(StorageError::WriteNotConfirmed)));

    let race = postgres::create_session(&mut admin, id(base + 6), identity(1), limits())
        .await
        .unwrap();
    let mut reader = connect("postgres").await;
    let mut writer = connect("postgres").await;
    let mut witness = connect("postgres").await;
    let expected_pids = [pid(&mut reader).await, pid(&mut writer).await];
    let mut guard = admin.begin().await.unwrap();
    sqlx::query("SELECT id FROM identity_access.sessions WHERE id = $1::text::numeric FOR UPDATE")
        .bind((base + 6).to_string())
        .execute(&mut *guard)
        .await
        .unwrap();
    let (checked, revoked, ()) = tokio::join!(
        postgres::record_activity(&mut reader, &race),
        postgres::revoke(&mut writer, id(base + 6)),
        async {
            waiters(&mut witness, &expected_pids).await;
            sqlx::query("UPDATE identity_access.sessions SET state = 'revoked' WHERE id = $1::text::numeric")
                .bind((base + 6).to_string()).execute(&mut *guard).await.unwrap();
            guard.commit().await.unwrap();
        }
    );
    assert!(checked.unwrap().is_none());
    assert!(revoked.unwrap());
    assert!(
        postgres::inspect(&mut admin, &race)
            .await
            .unwrap()
            .is_none()
    );

    // The blocked reader must use time after the lock, not the transaction-start timestamp.
    let crossing = SessionToken::generate().unwrap();
    let time = seconds(&mut admin).await;
    let deadline = time + 3;
    seed(
        &mut admin,
        base + 7,
        &crossing,
        Times {
            created: time - 2,
            activity: time - 2,
            observed: time - 2,
            absolute: deadline,
            idle: 100,
        },
    )
    .await;
    let reader_pid = pid(&mut reader).await;
    let mut guard = admin.begin().await.unwrap();
    sqlx::query("SELECT id FROM identity_access.sessions WHERE id = $1::text::numeric FOR UPDATE")
        .bind((base + 7).to_string())
        .execute(&mut *guard)
        .await
        .unwrap();
    let (checked, ()) = tokio::join!(postgres::inspect(&mut reader, &crossing), async {
        waiters(&mut witness, &[reader_pid]).await;
        let began_before_expiry: bool = sqlx::query_scalar("SELECT EXTRACT(epoch FROM xact_start) < $2::text::numeric FROM pg_catalog.pg_stat_activity WHERE pid = $1")
            .bind(reader_pid).bind(deadline.to_string()).fetch_one(&mut witness).await.unwrap();
        assert!(
            began_before_expiry,
            "reader must begin before expiry to test time after a lock wait"
        );
        tokio::time::timeout(Duration::from_secs(5), async {
            while seconds(&mut witness).await < deadline {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        guard.commit().await.unwrap();
    });
    assert!(checked.unwrap().is_none());
    assert_eq!(state(&mut admin, base + 7).await, "expired");

    let mut runtime = connect("kc_identity_test_runtime").await;
    assert!(
        postgres::inspect(&mut runtime, &activity)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!postgres::revoke(&mut runtime, id(base + 2)).await.unwrap());
    database_code(
        postgres::create_session(&mut runtime, id(base + 90), identity(1), limits())
            .await
            .unwrap_err(),
        "42501",
    );
    database_code(
        postgres::register_identity(&mut runtime, identity(90))
            .await
            .unwrap_err(),
        "42501",
    );
    for sql in [
        "SET ROLE kc_identity_owner",
        "SET ROLE kc_identity_migrator",
        "CREATE TABLE identity_access.unexpected (id integer)",
        "TRUNCATE identity_access.sessions",
        "DELETE FROM identity_access.sessions",
    ] {
        rejected(&mut runtime, sql, "42501").await;
    }
    let safe: bool = sqlx::query_scalar("SELECT NOT (rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls OR rolcanlogin) AND NOT pg_has_role('kc_identity_runtime', 'kc_identity_owner', 'MEMBER') AND NOT has_schema_privilege('kc_identity_runtime', 'identity_access', 'CREATE') AND NOT has_function_privilege('kc_identity_runtime', 'identity_access.guard_session_transition()', 'EXECUTE') FROM pg_roles WHERE rolname = 'kc_identity_runtime'")
        .fetch_one(&mut admin).await.unwrap();
    assert!(safe);

    // A real lost database connection must return an error, never a cached positive result.
    let reader_pid = pid(&mut reader).await;
    sqlx::query("SELECT pg_terminate_backend($1)")
        .bind(reader_pid)
        .execute(&mut admin)
        .await
        .unwrap();
    assert!(matches!(
        postgres::inspect(&mut reader, &activity).await,
        Err(StorageError::Database(_))
    ));
    let fingerprints_valid: bool = sqlx::query_scalar(
        "SELECT bool_and(octet_length(fingerprint) = 32) FROM identity_access.sessions",
    )
    .fetch_one(&mut admin)
    .await
    .unwrap();
    assert!(fingerprints_valid);
}
