use kitaconnect_organization_config::{FacilityId, provisioning};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{Connection, PgConnection, PgPool};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

const OWN: &str = "facility_00000000000000000000000000000001";

fn options(user: &str) -> PgConnectOptions {
    PgConnectOptions::new()
        .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
        .username(user)
        .database("kc_facility_schema_test")
        .ssl_mode(PgSslMode::Disable)
}

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(&options(user)).await.unwrap()
}

async fn execute(connection: &mut PgConnection, statement: &'static str) {
    sqlx::raw_sql(statement).execute(connection).await.unwrap();
}

async fn denied(connection: &mut PgConnection, statement: &'static str) {
    let error = sqlx::query(statement)
        .execute(connection)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
}

// Reference fixture only. This does not create a production pool or identity binding.
async fn test_pool(discard: Arc<AtomicBool>) -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .max_lifetime(None)
        .idle_timeout(None)
        .before_acquire(|connection, _| Box::pin(async move {
            sqlx::query_scalar("SELECT current_user = 'kc_test_care_1' AND session_user = 'kc_test_care_1' AND NOT (rolsuper OR rolbypassrls) FROM pg_catalog.pg_roles WHERE rolname = current_user")
                .fetch_one(connection).await
        }))
        .after_release(move |connection, _| {
            let discard = discard.clone();
            Box::pin(async move {
                if discard.swap(false, Ordering::SeqCst) {
                    return Err(sqlx::Error::Protocol("injected fixture reset failure".into()));
                }
                sqlx::raw_sql("RESET ROLE; RESET ALL").execute(connection).await?;
                Ok(true)
            })
        })
        .connect_with(options("kc_test_care_1"))
        .await.unwrap()
}

async fn clean(connection: &mut PgConnection, expected_pid: i32) {
    let (pid, safe, payload): (i32, bool, i32) = sqlx::query_as(
        "SELECT pg_catalog.pg_backend_pid(), current_user = 'kc_test_care_1' AND session_user = current_user \
         AND NULLIF(pg_catalog.current_setting('kc.facility_id', true), '') IS NULL \
         AND pg_catalog.current_setting('search_path') = '\"$user\", public', \
         (SELECT payload FROM facility_00000000000000000000000000000001.care_probe WHERE id = 1)",
    ).fetch_one(&mut *connection).await.unwrap();
    assert_eq!(pid, expected_pid, "new connection concealed pool reuse");
    assert!(safe, "previous session context survived pool return");
    assert_eq!(payload, 100, "cancelled operation persisted a write");
    denied(connection, "SET ROLE kc_test_care_2").await;
    denied(
        connection,
        "SELECT * FROM facility_00000000000000000000000000000002.care_probe",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn reused_pool_clears_context_and_observes_revocation() {
    let discard = Arc::new(AtomicBool::new(false));
    let pool = test_pool(discard.clone()).await;
    let mut connection = pool.acquire().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_catalog.pg_backend_pid()")
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    clean(&mut connection, pid).await;
    execute(
        &mut connection,
        "SET kc.facility_id = '2'; SET search_path = kc_test_shadow",
    )
    .await;
    drop(connection);

    let mut connection = pool.acquire().await.unwrap();
    clean(&mut connection, pid).await;
    execute(
        &mut connection,
        "SET kc.facility_id = '2'; SET search_path = kc_test_shadow",
    )
    .await;
    let error = sqlx::query("SELECT 1 / 0")
        .execute(&mut *connection)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("22012")
    );
    drop(connection);

    let mut connection = pool.acquire().await.unwrap();
    clean(&mut connection, pid).await;
    let mut transaction = connection.begin().await.unwrap();
    execute(&mut transaction, "SET LOCAL kc.facility_id = '2'; SET LOCAL search_path = kc_test_shadow; UPDATE facility_00000000000000000000000000000001.care_probe SET payload = 777 WHERE id = 1").await;
    // Cancellation happens after SQL context/write setup, while the transaction is still held.
    let cancelled = async move {
        let _held_transaction = transaction;
        std::future::pending::<()>().await;
    };
    assert!(
        tokio::time::timeout(Duration::from_millis(5), cancelled)
            .await
            .is_err()
    );
    drop(connection);

    let mut connection = pool.acquire().await.unwrap();
    clean(&mut connection, pid).await;
    let mut transaction = connection.begin().await.unwrap();
    execute(&mut transaction, "SET LOCAL kc.facility_id = '2'; SET LOCAL statement_timeout = '50ms'; UPDATE facility_00000000000000000000000000000001.care_probe SET payload = 888 WHERE id = 1").await;
    let error = sqlx::query("SELECT pg_catalog.pg_sleep(10)")
        .execute(&mut *transaction)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("57014")
    );
    drop(transaction); // SQLx queues ROLLBACK before the next asynchronous operation.
    drop(connection);

    let mut connection = pool.acquire().await.unwrap();
    clean(&mut connection, pid).await;
    let mut admin = connect("postgres").await;
    execute(
        &mut admin,
        "REVOKE USAGE ON SCHEMA facility_00000000000000000000000000000001 FROM kc_test_care_1",
    )
    .await;
    denied(
        &mut connection,
        "SELECT * FROM facility_00000000000000000000000000000001.care_probe",
    )
    .await;
    execute(
        &mut admin,
        "GRANT USAGE ON SCHEMA facility_00000000000000000000000000000001 TO kc_test_care_1",
    )
    .await;
    clean(&mut connection, pid).await;
    execute(&mut admin, "ALTER POLICY fixture_identity ON facility_00000000000000000000000000000001.care_probe TO kc_test_unmapped").await;
    let visible: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM facility_00000000000000000000000000000001.care_probe",
    )
    .fetch_one(&mut *connection)
    .await
    .unwrap();
    assert_eq!(visible, 0, "old connection ignored revoked policy mapping");
    denied(
        &mut connection,
        "INSERT INTO facility_00000000000000000000000000000001.care_probe VALUES (7, 1, 700)",
    )
    .await;
    execute(&mut admin, "ALTER POLICY fixture_identity ON facility_00000000000000000000000000000001.care_probe TO kc_test_care_1").await;
    clean(&mut connection, pid).await;

    let mut maintenance = connect("kc_facility_test_maintenance").await;
    assert!(
        provisioning::resolve(&mut maintenance, FacilityId::new(999).unwrap())
            .await
            .unwrap()
            .is_none()
    );
    let facility = FacilityId::new(1).unwrap();
    assert_eq!(
        provisioning::resolve(&mut maintenance, facility)
            .await
            .unwrap()
            .unwrap()
            .name(),
        OWN
    );
    execute(
        &mut admin,
        "DELETE FROM organization_config.facility_schemas WHERE facility_id = 1",
    )
    .await;
    assert!(
        provisioning::resolve(&mut maintenance, facility)
            .await
            .unwrap()
            .is_none()
    );
    execute(&mut admin, "INSERT INTO organization_config.facility_schemas VALUES (1, 'facility_00000000000000000000000000000001')").await;
    assert_eq!(
        provisioning::resolve(&mut maintenance, facility)
            .await
            .unwrap()
            .unwrap()
            .name(),
        OWN
    );
    clean(&mut connection, pid).await;

    // A reset failure must discard this physical connection, rather than return it dirty.
    discard.store(true, Ordering::SeqCst);
    drop(connection);
    let mut replacement = pool.acquire().await.unwrap();
    let replacement_pid: i32 = sqlx::query_scalar("SELECT pg_catalog.pg_backend_pid()")
        .fetch_one(&mut *replacement)
        .await
        .unwrap();
    assert_ne!(replacement_pid, pid);
    clean(&mut replacement, replacement_pid).await;
    drop(replacement);
    pool.close().await;
}
