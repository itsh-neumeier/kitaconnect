use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{AssertSqlSafe, Connection, PgConnection};

const ARTIFACT: &str = include_str!("../database/tests/service-isolation.sql");

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

// Dynamic identifiers below come exclusively from the fixed fixture matrix, never a client.
async fn execute(connection: &mut PgConnection, statement: &str) {
    sqlx::raw_sql(AssertSqlSafe(statement))
        .execute(connection)
        .await
        .unwrap();
}

async fn denied(connection: &mut PgConnection, statement: &str) {
    let error = sqlx::query(AssertSqlSafe(statement))
        .execute(connection)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501"),
        "unexpected failure for {statement}: {error}"
    );
}

async fn count(connection: &mut PgConnection, statement: &str) -> i64 {
    sqlx::query_scalar(AssertSqlSafe(statement))
        .fetch_one(connection)
        .await
        .unwrap()
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn service_objects_enforce_roles_and_rls() {
    let phase = std::env::var("KC_ORG_TEST_PHASE").expect("phase required");
    assert!(matches!(phase.as_str(), "before" | "after"));
    let mut admin = connect("postgres").await;
    if phase == "before" {
        let mut transaction = admin.begin().await.unwrap();
        sqlx::raw_sql(ARTIFACT)
            .execute(&mut *transaction)
            .await
            .unwrap();
        let error = sqlx::query("SELECT 1 / 0")
            .execute(&mut *transaction)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("22012")
        );
        transaction.rollback().await.unwrap();
        assert_eq!(
            count(
                &mut admin,
                "SELECT count(*) FROM pg_catalog.pg_roles WHERE rolname LIKE 'kc_test_%'"
            )
            .await,
            0
        );
        assert_eq!(count(&mut admin, "SELECT count(*) FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'facility_00000000000000000000000000000001'").await, 0);
        let restored: bool = sqlx::query_scalar("SELECT pg_catalog.has_database_privilege('kc_org_adapter_runtime', current_database(), 'TEMPORARY')")
            .fetch_one(&mut admin).await.unwrap();
        assert!(restored, "failed artifact must restore database grants");
        let mut transaction = admin.begin().await.unwrap();
        sqlx::raw_sql(ARTIFACT)
            .execute(&mut *transaction)
            .await
            .unwrap();
        transaction.commit().await.unwrap();
    }

    for facility in 1..=2 {
        let schema = format!("facility_{facility:032x}");
        let foreign_schema = format!("facility_{:032x}", 3 - facility);
        // Exact relation/function lists reject unnoticed extra service objects.
        let tables: Vec<String> = sqlx::query_scalar("SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = $1 AND c.relkind = 'r' ORDER BY c.relname")
            .bind(&schema).fetch_all(&mut admin).await.unwrap();
        assert_eq!(tables, ["care_probe", "meals_probe"]);
        let functions: Vec<String> = sqlx::query_scalar("SELECT p.proname::text FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = $1 ORDER BY p.proname")
            .bind(&schema).fetch_all(&mut admin).await.unwrap();
        assert_eq!(functions, ["care_probe_count", "meals_probe_count"]);

        for (service, foreign_service) in [("care", "meals"), ("meals", "care")] {
            let login = format!("kc_test_{service}_{facility}");
            let owner = format!("kc_test_{service}_owner");
            let migrator = format!("kc_test_{service}_migrator");
            let table = format!("{schema}.{service}_probe");
            let function = format!("{schema}.{service}_probe_count()");
            let mut runtime = connect(&login).await;
            let safe: bool = sqlx::query_scalar("SELECT NOT (rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls) AND NOT EXISTS (SELECT FROM pg_catalog.pg_auth_members WHERE member = r.oid) AND NOT EXISTS (SELECT FROM pg_catalog.pg_shdepend WHERE refobjid = r.oid AND deptype = 'o') AND NOT pg_catalog.has_database_privilege(current_user, current_database(), 'CREATE') AND NOT pg_catalog.has_database_privilege(current_user, current_database(), 'TEMPORARY') FROM pg_catalog.pg_roles r WHERE rolname = current_user")
                .fetch_one(&mut runtime).await.unwrap();
            assert!(safe, "unsafe fixture runtime identity {login}");
            let table_safe: bool = sqlx::query_scalar("SELECT c.relrowsecurity AND c.relforcerowsecurity AND pg_catalog.pg_get_userbyid(c.relowner) = $2 FROM pg_catalog.pg_class c WHERE c.oid = $1::regclass")
                .bind(&table).bind(&owner).fetch_one(&mut admin).await.unwrap();
            assert!(table_safe);
            let function_safe: bool = sqlx::query_scalar("SELECT NOT p.prosecdef AND pg_catalog.pg_get_userbyid(p.proowner) = $2 AND p.proconfig = ARRAY['search_path=pg_catalog, pg_temp'] FROM pg_catalog.pg_proc p WHERE p.oid = $1::regprocedure")
                .bind(&function).bind(&owner).fetch_one(&mut admin).await.unwrap();
            assert!(function_safe);
            assert_eq!(
                count(&mut runtime, &format!("SELECT count(*) FROM {table}")).await,
                1
            );
            assert_eq!(count(&mut runtime, &format!("SELECT {function}")).await, 1);
            execute(
                &mut runtime,
                &format!("SET kc.facility_id = '{}'; RESET ROLE", 3 - facility),
            )
            .await;
            assert_eq!(
                count(&mut runtime, &format!("SELECT count(*) FROM {table}")).await,
                1
            );

            for statement in [
                format!("SELECT * FROM {schema}.{foreign_service}_probe"),
                format!("SELECT * FROM {foreign_schema}.{service}_probe"),
                format!("SELECT {schema}.{foreign_service}_probe_count()"),
                format!("SET ROLE {owner}"),
                format!("SET ROLE {migrator}"),
                format!("SET ROLE kc_test_{service}_{}", 3 - facility),
                "SET SESSION AUTHORIZATION postgres".into(),
                format!("CREATE TABLE {schema}.unexpected (id integer)"),
                "CREATE TEMP TABLE unexpected (id integer)".into(),
                "CREATE TABLE public.unexpected (id integer)".into(),
                format!("ALTER TABLE {table} OWNER TO {login}"),
                format!("ALTER TABLE {table} DISABLE ROW LEVEL SECURITY"),
                format!("ALTER FUNCTION {function} OWNER TO {login}"),
                format!(
                    "CREATE OR REPLACE FUNCTION {function} RETURNS bigint LANGUAGE sql AS 'SELECT 999::bigint'"
                ),
                format!("TRUNCATE {table}"),
                format!("ALTER ROLE {login} BYPASSRLS"),
                format!("GRANT {owner} TO {login}"),
                format!("INSERT INTO {table} VALUES (4, {}, 400)", 3 - facility),
                format!(
                    "UPDATE {table} SET facility_id = {} WHERE id = 1",
                    3 - facility
                ),
            ] {
                denied(&mut runtime, &statement).await;
            }
            // PostgreSQL may warn rather than error for GRANT without a grant option.
            execute(&mut runtime, &format!("GRANT SELECT ON {table} TO PUBLIC")).await;
            let public_grant: bool = sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_catalog.pg_class c, LATERAL pg_catalog.aclexplode(c.relacl) a WHERE c.oid = $1::regclass AND a.grantee = 0)")
                .bind(&table).fetch_one(&mut admin).await.unwrap();
            assert!(!public_grant, "runtime granted PUBLIC access");
            let mut unmapped = connect("kc_test_unmapped").await;
            denied(&mut unmapped, &format!("SELECT * FROM {table}")).await;
            denied(&mut unmapped, &format!("SELECT {function}")).await;
            // Legitimate mutations work, while foreign rows cannot be updated or deleted.
            execute(
                &mut runtime,
                &format!("INSERT INTO {table} VALUES (3, {facility}, 300)"),
            )
            .await;
            let changed = sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET payload = 301 WHERE id = 3"
            )))
            .execute(&mut runtime)
            .await
            .unwrap()
            .rows_affected();
            assert_eq!(changed, 1);
            for statement in [
                format!("UPDATE {table} SET payload = 0 WHERE id = 2"),
                format!("DELETE FROM {table} WHERE id = 2"),
            ] {
                assert_eq!(
                    sqlx::query(AssertSqlSafe(statement))
                        .execute(&mut runtime)
                        .await
                        .unwrap()
                        .rows_affected(),
                    0
                );
            }
            execute(&mut runtime, &format!("DELETE FROM {table} WHERE id = 3")).await;
            execute(&mut runtime, "SET row_security = off").await;
            denied(&mut runtime, &format!("SELECT * FROM {table}")).await;
            execute(&mut runtime, "SET row_security = on").await;

            // FORCE RLS applies even to the actual table owner, without an owner policy.
            let mut migration = connect(&migrator).await;
            denied(&mut migration, &format!("SELECT * FROM {table}")).await;
            denied(
                &mut migration,
                &format!("SET ROLE kc_test_{foreign_service}_owner"),
            )
            .await;
            execute(&mut migration, &format!("SET ROLE {owner}")).await;
            assert_eq!(
                count(&mut migration, &format!("SELECT count(*) FROM {table}")).await,
                0
            );
            denied(
                &mut migration,
                &format!("INSERT INTO {table} VALUES (4, {facility}, 400)"),
            )
            .await;
            denied(
                &mut migration,
                &format!("CREATE TABLE {schema}.unexpected (id integer)"),
            )
            .await;
            denied(
                &mut migration,
                &format!("SELECT * FROM {schema}.{foreign_service}_probe"),
            )
            .await;
            denied(
                &mut migration,
                &format!("ALTER TABLE {table} OWNER TO kc_test_{foreign_service}_owner"),
            )
            .await;
            // Privileged inspection confirms foreign rows survived unchanged.
            assert_eq!(
                count(
                    &mut admin,
                    &format!("SELECT payload::bigint FROM {table} WHERE id = 2")
                )
                .await,
                900
            );
        }
    }

    let mut runtime = connect("kc_test_care_1").await;
    execute(
        &mut runtime,
        "SET search_path = kc_test_shadow, facility_00000000000000000000000000000002, pg_temp",
    )
    .await;
    // Demonstrate that the hostile path is active, then verify both qualified accesses.
    assert_eq!(count(&mut runtime, "SELECT care_probe_count()").await, 999);
    assert_eq!(
        count(&mut runtime, "SELECT payload::bigint FROM care_probe").await,
        999
    );
    assert_eq!(
        count(
            &mut runtime,
            "SELECT facility_00000000000000000000000000000001.care_probe_count()"
        )
        .await,
        1
    );
    assert_eq!(count(&mut runtime, "SELECT payload::bigint FROM facility_00000000000000000000000000000001.care_probe WHERE id = 1").await, 100);
    denied(&mut runtime, "CREATE FUNCTION kc_test_shadow.care_probe_count(integer) RETURNS bigint LANGUAGE sql AS 'SELECT 0::bigint'").await;
    let mut blocked = connect("kc_org_adapter_runtime").await;
    denied(
        &mut blocked,
        "SELECT facility_00000000000000000000000000000001.care_probe_count()",
    )
    .await;
    denied(
        &mut blocked,
        "SELECT * FROM facility_00000000000000000000000000000001.care_probe",
    )
    .await;
}
