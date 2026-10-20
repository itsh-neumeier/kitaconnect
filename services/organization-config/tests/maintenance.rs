use kitaconnect_organization_config::{
    FacilityId,
    provisioning::{self, ProvisionError},
};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use std::time::Duration;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
            .username(user)
            .database("kc_facility_schema_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}

fn facility(value: u128) -> FacilityId {
    FacilityId::new(value).unwrap()
}

async fn execute(connection: &mut PgConnection, sql: &str) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(AssertSqlSafe(sql))
        .execute(connection)
        .await?;
    Ok(())
}

async fn wait_for_two_waiters(witness: &mut PgConnection) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_locks WHERE locktype = 'advisory' AND classid = 184005 AND objid = 1 AND NOT granted AND database = (SELECT oid FROM pg_catalog.pg_database WHERE datname = current_database())")
                .fetch_one(&mut *witness).await.unwrap();
            if waiting == 2 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("both real database operations must wait at the maintenance lock");
}

// Test-only, fixed technical versions. No production orchestrator or arbitrary artifact input.
async fn migration_step(
    connection: &mut PgConnection,
    id: FacilityId,
    target: i32,
    fail: bool,
) -> Result<bool, sqlx::Error> {
    assert!(matches!(target, 1 | 2));
    let mut transaction = connection.begin().await?;
    let result: Result<bool, sqlx::Error> = async {
        execute(
            &mut transaction,
            "SELECT pg_catalog.pg_advisory_xact_lock(184005, 1)",
        )
        .await?;
        let schema = provisioning::resolve(&mut transaction, id)
            .await
            .unwrap()
            .expect("registered fixture schema");
        let versions = format!("{}.care_wave_versions", schema.name());
        let probe = format!("{}.care_wave_probe", schema.name());
        let exists: bool = sqlx::query_scalar("SELECT pg_catalog.to_regclass($1) IS NOT NULL")
            .bind(&versions)
            .fetch_one(&mut *transaction)
            .await?;
        let current = if exists {
            sqlx::query_scalar::<_, i32>(AssertSqlSafe(format!(
                "SELECT max(version) FROM {versions}"
            )))
            .fetch_one(&mut *transaction)
            .await?
        } else {
            0
        };
        if current == target {
            return Ok(false);
        }
        assert_eq!(current, target - 1, "fixture version precondition");
        if target == 1 {
            execute(
                &mut transaction,
                &format!(
                    "CREATE TABLE {versions} (version integer PRIMARY KEY); \
                 CREATE TABLE {probe} (value integer NOT NULL); \
                 ALTER TABLE {versions} OWNER TO kc_test_care_owner; \
                 ALTER TABLE {probe} OWNER TO kc_test_care_owner; \
                 ALTER TABLE {versions} ENABLE ROW LEVEL SECURITY; \
                 ALTER TABLE {versions} FORCE ROW LEVEL SECURITY; \
                 ALTER TABLE {probe} ENABLE ROW LEVEL SECURITY; \
                 ALTER TABLE {probe} FORCE ROW LEVEL SECURITY; \
                 INSERT INTO {probe} VALUES (100)"
                ),
            )
            .await?;
        } else {
            execute(
                &mut transaction,
                &format!("ALTER TABLE {probe} ADD COLUMN marker integer NOT NULL DEFAULT 2"),
            )
            .await?;
        }
        execute(
            &mut transaction,
            &format!("INSERT INTO {versions} VALUES ({target})"),
        )
        .await?;
        if fail {
            execute(&mut transaction, "SELECT 1 / 0").await?;
        }
        Ok(true)
    }
    .await;
    match result {
        Ok(changed) => {
            transaction.commit().await?;
            Ok(changed)
        }
        Err(error) => {
            transaction.rollback().await?;
            Err(error)
        }
    }
}

async fn versions(connection: &mut PgConnection, id: u128) -> Option<Vec<i32>> {
    let name = format!("facility_{id:032x}.care_wave_versions");
    let exists: bool = sqlx::query_scalar("SELECT pg_catalog.to_regclass($1) IS NOT NULL")
        .bind(&name)
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    if !exists {
        return None;
    }
    Some(
        sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT version FROM {name} ORDER BY version"
        )))
        .fetch_all(connection)
        .await
        .unwrap(),
    )
}

async fn marker_exists(connection: &mut PgConnection, id: u128) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_catalog.pg_attribute a JOIN pg_catalog.pg_class c ON c.oid = a.attrelid JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1 AND c.relname = 'care_wave_probe' AND a.attname = 'marker' AND NOT a.attisdropped)")
        .bind(format!("facility_{id:032x}")).fetch_one(connection).await.unwrap()
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn concurrent_maintenance_preserves_registry_and_wave_versions() {
    let phase = std::env::var("KC_ORG_TEST_PHASE").expect("phase required");
    assert!(matches!(phase.as_str(), "before" | "after"));
    let mut admin = connect("postgres").await;
    let mut guard_connection = connect("postgres").await;
    let mut witness = connect("postgres").await;
    if phase == "before" {
        execute(
            &mut admin,
            "INSERT INTO organization_config.facilities (id) VALUES (10), (11), (12)",
        )
        .await
        .unwrap();
        let mut left = connect("kc_facility_test_maintenance").await;
        let mut right = connect("kc_facility_test_maintenance").await;
        let mut guard = guard_connection.begin().await.unwrap();
        execute(
            &mut guard,
            "SELECT pg_catalog.pg_advisory_xact_lock(184005, 1)",
        )
        .await
        .unwrap();
        let (a, b, ()) = tokio::join!(
            provisioning::provision(&mut left, facility(10)),
            provisioning::provision(&mut right, facility(10)),
            async {
                wait_for_two_waiters(&mut witness).await;
                guard.commit().await.unwrap();
            }
        );
        assert!(matches!(
            (&a, &b),
            (Ok(_), Err(ProvisionError::AlreadyRegistered))
                | (Err(ProvisionError::AlreadyRegistered), Ok(_))
        ));
        provisioning::provision(&mut left, facility(11))
            .await
            .unwrap();

        let mut left = connect("postgres").await;
        let mut right = connect("postgres").await;
        let mut guard = guard_connection.begin().await.unwrap();
        execute(
            &mut guard,
            "SELECT pg_catalog.pg_advisory_xact_lock(184005, 1)",
        )
        .await
        .unwrap();
        let (a, b, ()) = tokio::join!(
            migration_step(&mut left, facility(10), 1, false),
            migration_step(&mut right, facility(10), 1, false),
            async {
                wait_for_two_waiters(&mut witness).await;
                guard.commit().await.unwrap();
            }
        );
        assert_ne!(
            a.unwrap(),
            b.unwrap(),
            "exactly one version step must be applied"
        );
        assert_eq!(versions(&mut admin, 10).await, Some(vec![1]));

        // Provisioning and a migration share the same lock, not merely separate test locks.
        let mut provisioner = connect("kc_facility_test_maintenance").await;
        let mut guard = guard_connection.begin().await.unwrap();
        execute(
            &mut guard,
            "SELECT pg_catalog.pg_advisory_xact_lock(184005, 1)",
        )
        .await
        .unwrap();
        let (created, migrated, ()) = tokio::join!(
            provisioning::provision(&mut provisioner, facility(12)),
            migration_step(&mut left, facility(11), 1, false),
            async {
                wait_for_two_waiters(&mut witness).await;
                guard.commit().await.unwrap();
            }
        );
        created.unwrap();
        assert!(migrated.unwrap());

        let mut completed = Vec::new();
        for id in [10, 11, 12] {
            match migration_step(&mut admin, facility(id), 2, id == 11).await {
                Ok(true) => completed.push(id),
                Ok(false) => panic!("wave step was unexpectedly applied already"),
                Err(error) => {
                    assert_eq!(id, 11);
                    assert_eq!(
                        error.as_database_error().unwrap().code().as_deref(),
                        Some("22012")
                    );
                    break; // The next facility must not be visited after failure.
                }
            }
        }
        assert_eq!(completed, [10]);
        assert_eq!(versions(&mut admin, 10).await, Some(vec![1, 2]));
        assert_eq!(versions(&mut admin, 11).await, Some(vec![1]));
        assert!(
            !marker_exists(&mut admin, 11).await,
            "failed DDL survived rollback"
        );
        assert_eq!(versions(&mut admin, 12).await, None);
        assert!(
            migration_step(&mut admin, facility(11), 2, false)
                .await
                .unwrap()
        );
    }

    for id in [10, 11, 12] {
        let expected = format!("facility_{id:032x}");
        let mut maintenance = connect("kc_facility_test_maintenance").await;
        assert_eq!(
            provisioning::resolve(&mut maintenance, facility(id))
                .await
                .unwrap()
                .unwrap()
                .name(),
            expected
        );
        let records: i64 = sqlx::query_scalar("SELECT count(*) FROM organization_config.facility_schemas WHERE facility_id = $1::text::numeric")
            .bind(id.to_string()).fetch_one(&mut admin).await.unwrap();
        assert_eq!(records, 1);
        if id != 12 {
            assert_eq!(versions(&mut admin, id).await, Some(vec![1, 2]));
            assert!(marker_exists(&mut admin, id).await);
            let runtime_blocked: bool = sqlx::query_scalar(
                "SELECT NOT pg_catalog.has_schema_privilege('kc_test_care_1', $1, 'USAGE')",
            )
            .bind(&expected)
            .fetch_one(&mut admin)
            .await
            .unwrap();
            assert!(runtime_blocked);
        } else {
            assert_eq!(versions(&mut admin, id).await, None);
        }
    }
}
