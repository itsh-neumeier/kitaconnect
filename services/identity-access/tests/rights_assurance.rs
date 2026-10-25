use kitaconnect_identity_access::{
    IdentityId,
    login::LoginFacilityId,
    rights_assurance::{self, CurrentAssurance},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;

const BASIC: &str =
    r#"{"format":1,"roles":[{"key":"synthetic","operations":["basic_contact_data"]}]}"#;
const SENSITIVE: &str =
    r#"{"format":1,"roles":[{"key":"synthetic","operations":["health_data"]}]}"#;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated test socket required"))
            .username(user)
            .database("kc_role_assurance_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn read(
    c: &mut PgConnection,
    identity: u128,
    facility: u128,
) -> Result<CurrentAssurance, rights_assurance::ReadError> {
    rights_assurance::read_current(
        c,
        IdentityId::new(identity).unwrap(),
        LoginFacilityId::new(facility).unwrap(),
    )
    .await
}
async fn sql(c: &mut PgConnection, text: &str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn rejected(c: &mut PgConnection, text: &str, code: &str) {
    let error = sqlx::raw_sql(text).execute(c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some(code)
    );
}
async fn insert(c: &mut PgConnection, identity: u128, facility: u128, document: &str) {
    sqlx::query("INSERT INTO identity_access.role_assurance_scopes VALUES ($1::text::numeric,$2::text::numeric,1,'active',$3)")
        .bind(identity.to_string()).bind(facility.to_string()).bind(document).execute(c).await.unwrap();
}

#[tokio::test]
#[ignore = "requires the isolated PostgreSQL fixture"]
async fn current_roles_are_complete_versioned_and_restricted() {
    let mut root = connect("postgres").await;
    let mut runtime = connect("kc_role_assurance_runtime").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        let persisted = read(&mut root, 1, 10).await.unwrap();
        assert_eq!(persisted.version(), 3);
        assert!(!persisted.mfa_required());
        assert!(read(&mut root, 2, 10).await.is_err());
        assert!(read(&mut runtime, 1, 10).await.is_err());
        // Recreate only synthetic fixtures after first proving persisted state.
        sql(&mut root, "TRUNCATE identity_access.role_assurance_scopes").await;
    } else {
        sql(
            &mut root,
            "INSERT INTO identity_access.identities SELECT generate_series(1,8)",
        )
        .await;
    }
    insert(&mut root, 1, 10, BASIC).await;
    insert(&mut root, 2, 10, SENSITIVE).await;
    insert(&mut root, 3, 10, r#"{"format":1,"roles":[]}"#).await;
    insert(
        &mut root,
        4,
        10,
        r#"{"format":1,"roles":[{"key":"synthetic","operations":["unknown"]}]}"#,
    )
    .await;
    insert(&mut root, 5, 10, r#"{"format":1,"roles":[],"roles":[]}"#).await;
    assert!(read(&mut runtime, 1, 10).await.is_err());
    for (identity, facility) in [(8, 10), (1, 20), (4, 10), (5, 10)] {
        assert!(read(&mut root, identity, facility).await.is_err());
    }
    assert!(!read(&mut root, 3, 10).await.unwrap().mfa_required());
    let value = read(&mut root, 1, 10).await.unwrap();
    assert_eq!(value.identity().value(), 1);
    assert_eq!(value.facility().value(), 10);
    assert_eq!(value.version(), 1);
    assert!(!value.mfa_required());
    assert!(read(&mut root, 2, 10).await.unwrap().mfa_required());

    rejected(
        &mut root,
        "UPDATE identity_access.role_assurance_scopes SET document='{}' WHERE identity_id=1",
        "23514",
    )
    .await;
    rejected(&mut root, "UPDATE identity_access.role_assurance_scopes SET facility_id=20,version=2 WHERE identity_id=1", "23514").await;
    rejected(
        &mut root,
        "DELETE FROM identity_access.role_assurance_scopes WHERE identity_id=1",
        "23514",
    )
    .await;
    rejected(
        &mut root,
        "INSERT INTO identity_access.role_assurance_scopes VALUES (1,20,2,'active','{}')",
        "23514",
    )
    .await;
    rejected(
        &mut root,
        "INSERT INTO identity_access.role_assurance_scopes VALUES (999,10,1,'active','{}')",
        "23503",
    )
    .await;
    rejected(&mut root, "INSERT INTO identity_access.role_assurance_scopes VALUES (1,20,1,'active',repeat('x',65537))", "23514").await;
    rejected(
        &mut runtime,
        "UPDATE identity_access.role_assurance_scopes SET state='revoked',version=2",
        "42501",
    )
    .await;
    rejected(
        &mut runtime,
        "INSERT INTO identity_access.role_assurance_scopes VALUES (1,20,1,'active','{}')",
        "42501",
    )
    .await;

    sql(&mut root, "BEGIN ISOLATION LEVEL REPEATABLE READ").await;
    assert!(read(&mut root, 1, 10).await.is_err());
    sql(&mut root, "ROLLBACK").await;

    // A separate real backend holds an uncommitted replacement. The reader sees
    // the complete previous version, then the complete committed replacement.
    let mut writer = connect("postgres").await;
    sql(&mut writer, "BEGIN").await;
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=2 WHERE identity_id=1")
        .bind(SENSITIVE).execute(&mut writer).await.unwrap();
    let old = read(&mut root, 1, 10).await.unwrap();
    assert_eq!(old.version(), 1);
    assert!(!old.mfa_required());
    sql(&mut writer, "COMMIT").await;
    let current = read(&mut root, 1, 10).await.unwrap();
    assert_eq!(current.version(), 2);
    assert!(current.mfa_required());
    // Rollback must not replace the successful current version.
    sql(&mut writer, "BEGIN").await;
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=3 WHERE identity_id=1")
        .bind(BASIC).execute(&mut writer).await.unwrap();
    sql(&mut writer, "ROLLBACK").await;
    assert_eq!(read(&mut root, 1, 10).await.unwrap().version(), 2);

    // Two actual writers race with the same expected version. Exactly one CAS wins.
    sql(&mut writer, "BEGIN").await;
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=3 WHERE identity_id=1 AND version=2")
        .bind(BASIC).execute(&mut writer).await.unwrap();
    let mut contender = connect("postgres").await;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut contender)
        .await
        .unwrap();
    let waiter = tokio::spawn(async move {
        sqlx::query("UPDATE identity_access.role_assurance_scopes SET version=3 WHERE identity_id=1 AND version=2")
            .execute(&mut contender).await.unwrap().rows_affected()
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock' AND state='active')")
                .bind(pid).fetch_one(&mut root).await.unwrap();
            if waiting { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("real conflicting writer was not observed");
    sql(&mut writer, "COMMIT").await;
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), waiter)
            .await
            .unwrap()
            .unwrap(),
        0
    );
    assert_eq!(read(&mut root, 1, 10).await.unwrap().version(), 3);
    assert!(!read(&mut root, 1, 10).await.unwrap().mfa_required());

    sql(&mut root, "UPDATE identity_access.role_assurance_scopes SET state='revoked',version=2 WHERE identity_id=2").await;
    assert!(read(&mut root, 2, 10).await.is_err());
    rejected(&mut root, "UPDATE identity_access.role_assurance_scopes SET state='active',version=3 WHERE identity_id=2", "23514").await;

    // Test-only policy grants one entire scope. It cannot hide one embedded role.
    sql(&mut root, "CREATE POLICY synthetic_scope ON identity_access.role_assurance_scopes FOR SELECT TO kc_identity_runtime USING (identity_id=1 AND facility_id=10)").await;
    assert_eq!(read(&mut runtime, 1, 10).await.unwrap().version(), 3);
    assert!(read(&mut runtime, 3, 10).await.is_err());
    sql(
        &mut root,
        "DROP POLICY synthetic_scope ON identity_access.role_assurance_scopes",
    )
    .await;
    assert!(read(&mut runtime, 1, 10).await.is_err());

    // A real server-side access error is redacted and cannot fall back to false MFA.
    sql(
        &mut root,
        "REVOKE SELECT ON identity_access.role_assurance_scopes FROM kc_identity_runtime",
    )
    .await;
    let error = read(&mut runtime, 1, 10).await.unwrap_err();
    assert_eq!(format!("{error:?}"), "current role assurance unavailable");
    sql(
        &mut root,
        "GRANT SELECT ON identity_access.role_assurance_scopes TO kc_identity_runtime",
    )
    .await;
    assert!(read(&mut runtime, 1, 10).await.is_err());
}
