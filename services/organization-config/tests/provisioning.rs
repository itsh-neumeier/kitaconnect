use kitaconnect_organization_config::{FacilityId, provisioning};
use provisioning::ProvisionError;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{Connection, PgConnection};

fn facility(value: u128) -> FacilityId {
    FacilityId::new(value).unwrap()
}

async fn connect(user: &str) -> PgConnection {
    let socket = std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required");
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(socket)
            .username(user)
            .database("kc_facility_schema_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}

async fn execute(connection: &mut PgConnection, sql: &'static str) {
    sqlx::raw_sql(sql).execute(connection).await.unwrap();
}

async fn absent(connection: &mut PgConnection, name: &str) {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = $1)",
    )
    .bind(name)
    .fetch_one(connection)
    .await
    .unwrap();
    assert!(!exists, "failed provisioning left a schema behind");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated schema fixture in scripts/test-organization-db.mjs"]
async fn schema_provisioning_is_atomic_and_restricted() {
    let phase = std::env::var("KC_ORG_TEST_PHASE").expect("phase required");
    assert!(matches!(phase.as_str(), "before" | "after"));
    let mut admin = connect("postgres").await;
    let mut maintenance = connect("kc_facility_test_maintenance").await;
    if phase == "before" {
        assert_eq!(
            provisioning::resolve(&mut maintenance, facility(1))
                .await
                .unwrap(),
            None
        );
        assert!(matches!(
            provisioning::provision(&mut maintenance, facility(99)).await,
            Err(ProvisionError::UnknownFacility)
        ));
        absent(&mut admin, "facility_00000000000000000000000000000063").await;
        provisioning::provision(&mut maintenance, facility(1))
            .await
            .unwrap();
        provisioning::provision(&mut maintenance, facility(u128::MAX))
            .await
            .unwrap();
        assert!(matches!(
            provisioning::provision(&mut maintenance, facility(1)).await,
            Err(ProvisionError::AlreadyRegistered)
        ));

        // Privileged fixture mutations only: each unsafe global role state must block a new provision.
        for (make_unsafe, restore) in [
            (
                "ALTER ROLE kc_org_runtime LOGIN",
                "ALTER ROLE kc_org_runtime NOLOGIN",
            ),
            (
                "ALTER ROLE kc_org_runtime CREATEDB",
                "ALTER ROLE kc_org_runtime NOCREATEDB",
            ),
            (
                "ALTER ROLE kc_org_runtime CREATEROLE",
                "ALTER ROLE kc_org_runtime NOCREATEROLE",
            ),
            (
                "ALTER ROLE kc_org_runtime REPLICATION",
                "ALTER ROLE kc_org_runtime NOREPLICATION",
            ),
            (
                "GRANT CREATE ON DATABASE kc_facility_schema_test TO kc_org_runtime",
                "REVOKE CREATE ON DATABASE kc_facility_schema_test FROM kc_org_runtime",
            ),
            (
                "ALTER ROLE kc_org_runtime BYPASSRLS",
                "ALTER ROLE kc_org_runtime NOBYPASSRLS",
            ),
            (
                "ALTER ROLE kc_org_runtime SUPERUSER",
                "ALTER ROLE kc_org_runtime NOSUPERUSER",
            ),
            (
                "GRANT kc_org_owner TO kc_org_runtime WITH ADMIN FALSE, INHERIT FALSE, SET FALSE",
                "REVOKE kc_org_owner FROM kc_org_runtime",
            ),
            (
                "CREATE SCHEMA kc_test_runtime_owned AUTHORIZATION kc_org_runtime",
                "DROP SCHEMA kc_test_runtime_owned",
            ),
        ] {
            execute(&mut admin, make_unsafe).await;
            let result = provisioning::provision(&mut maintenance, facility(2)).await;
            execute(&mut admin, restore).await;
            assert!(matches!(result, Err(ProvisionError::UnsafeRuntimeRole)));
            absent(&mut admin, "facility_00000000000000000000000000000002").await;
        }

        // Refuse to adopt a preexisting schema even if its owner is the expected role.
        execute(&mut admin, "CREATE SCHEMA facility_00000000000000000000000000000003 AUTHORIZATION kc_facility_schema_owner").await;
        assert!(matches!(
            provisioning::provision(&mut maintenance, facility(3)).await,
            Err(ProvisionError::SchemaExists)
        ));
        assert_eq!(
            provisioning::resolve(&mut maintenance, facility(3))
                .await
                .unwrap(),
            None
        );
        execute(
            &mut admin,
            "DROP SCHEMA facility_00000000000000000000000000000003",
        )
        .await;

        // DDL must roll back if the later registry insert fails. No production fault hook is needed.
        execute(
            &mut admin,
            "REVOKE INSERT ON organization_config.facility_schemas FROM kc_facility_provisioner",
        )
        .await;
        let result = provisioning::provision(&mut maintenance, facility(2)).await;
        execute(
            &mut admin,
            "GRANT INSERT ON organization_config.facility_schemas TO kc_facility_provisioner",
        )
        .await;
        assert!(matches!(result, Err(ProvisionError::Database(_))));
        absent(&mut admin, "facility_00000000000000000000000000000002").await;
        assert_eq!(
            provisioning::resolve(&mut maintenance, facility(2))
                .await
                .unwrap(),
            None
        );
        provisioning::provision(&mut maintenance, facility(2))
            .await
            .unwrap();

        execute(&mut admin, "UPDATE organization_config.facility_schemas SET schema_name = 'facility_000000000000000000000000000000f0' WHERE facility_id = 1").await;
        assert!(matches!(
            provisioning::resolve(&mut maintenance, facility(1)).await,
            Err(ProvisionError::InvalidRegistration)
        ));
        execute(&mut admin, "UPDATE organization_config.facility_schemas SET schema_name = 'facility_00000000000000000000000000000001' WHERE facility_id = 1").await;
        execute(
            &mut admin,
            "ALTER SCHEMA facility_00000000000000000000000000000001 OWNER TO postgres",
        )
        .await;
        assert!(matches!(
            provisioning::resolve(&mut maintenance, facility(1)).await,
            Err(ProvisionError::InvalidRegistration)
        ));
        execute(&mut admin, "ALTER SCHEMA facility_00000000000000000000000000000001 OWNER TO kc_facility_schema_owner").await;
        execute(
            &mut admin,
            "GRANT USAGE ON SCHEMA facility_00000000000000000000000000000001 TO PUBLIC",
        )
        .await;
        assert!(matches!(
            provisioning::resolve(&mut maintenance, facility(1)).await,
            Err(ProvisionError::InvalidRegistration)
        ));
        execute(
            &mut admin,
            "REVOKE USAGE ON SCHEMA facility_00000000000000000000000000000001 FROM PUBLIC",
        )
        .await;
    }

    for (value, expected) in [
        (1, "facility_00000000000000000000000000000001"),
        (2, "facility_00000000000000000000000000000002"),
        (u128::MAX, "facility_ffffffffffffffffffffffffffffffff"),
    ] {
        assert_eq!(
            provisioning::resolve(&mut maintenance, facility(value))
                .await
                .unwrap()
                .unwrap()
                .name(),
            expected
        );
    }
    assert_eq!(
        provisioning::resolve(&mut maintenance, facility(3))
            .await
            .unwrap(),
        None
    );
    let version: Vec<i32> = sqlx::query_scalar(
        "SELECT version FROM organization_config.schema_versions ORDER BY version",
    )
    .fetch_all(&mut admin)
    .await
    .unwrap();
    assert_eq!(version, [1, 2]);

    for statement in [
        "SELECT carrier_id FROM organization_config.facilities",
        "UPDATE organization_config.facility_schemas SET schema_name = schema_name WHERE facility_id = 1",
        "DELETE FROM organization_config.facility_schemas WHERE facility_id = 1",
    ] {
        let error = sqlx::query(statement)
            .execute(&mut maintenance)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("42501")
        );
    }

    let mut runtime = connect("kc_org_adapter_runtime").await;
    assert!(matches!(
        provisioning::provision(&mut runtime, facility(3)).await,
        Err(ProvisionError::MaintenanceRequired)
    ));
    assert!(matches!(
        provisioning::resolve(&mut runtime, facility(1)).await,
        Err(ProvisionError::MaintenanceRequired)
    ));
    let error = sqlx::query("SELECT * FROM organization_config.facility_schemas")
        .fetch_all(&mut runtime)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    let error = sqlx::query(
        "CREATE TABLE facility_00000000000000000000000000000001.runtime_probe (id integer)",
    )
    .execute(&mut runtime)
    .await
    .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    absent(&mut admin, "facility_00000000000000000000000000000003").await;
}
