use kitaconnect_organization_config::{
    FacilityId,
    membership::{self, IdentityReference},
    postgres,
    session_policy::{self, SessionPolicyLimits},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;
fn identity(n: u128) -> IdentityReference {
    IdentityReference::new(n).unwrap()
}
fn facility(n: u128) -> FacilityId {
    FacilityId::new(n).unwrap()
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .database("kc_membership_test")
            .username(user)
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn reject(c: &mut PgConnection, text: &'static str, expected: &str) {
    let error = sqlx::query(text).execute(c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some(expected)
    );
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn memberships_are_current_and_read_only() {
    let mut c = connect("postgres").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        assert!(
            membership::read_current(&mut c, identity(1), facility(1))
                .await
                .unwrap()
                .is_none()
        );
        let old = membership::read_current(&mut c, identity(1), facility(2))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(old.policy().version().value(), 2);
        assert_eq!(old.policy().limits().idle_seconds(), 300);
    }
    let offset = if after { 100 } else { 0 };
    let person = identity(offset + 1);
    let a = facility(offset + 1);
    let b = facility(offset + 2);
    let no_policy = facility(offset + 3);
    for f in [a, b, no_policy] {
        postgres::register_facility(&mut c, f, None).await.unwrap();
    }
    for f in [a, b] {
        session_policy::write(&mut c, f, None, SessionPolicyLimits::default())
            .await
            .unwrap();
    }
    assert!(
        membership::read_current(&mut c, person, a)
            .await
            .unwrap()
            .is_none()
    );
    // Explicit synthetic administrator fixtures, not a public membership write API.
    for f in [a, b, no_policy] {
        sqlx::query("INSERT INTO organization_config.memberships(identity_ref,facility_id,version,active) VALUES($1::text::numeric,$2::text::numeric,1,true)")
        .bind(person.value().to_string()).bind(f.value().to_string()).execute(&mut c).await.unwrap();
    }
    let initial = membership::read_current(&mut c, person, a)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(initial.identity(), person);
    assert_eq!(initial.facility(), a);
    assert_eq!(initial.version(), 1);
    assert_eq!(initial.policy().version().value(), 1);
    assert_eq!(initial.policy().limits(), SessionPolicyLimits::default());
    assert!(
        membership::read_current(&mut c, identity(offset + 2), a)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        membership::read_current(&mut c, person, facility(offset + 4))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        membership::read_current(&mut c, person, no_policy)
            .await
            .unwrap()
            .is_none()
    );
    let n:i64=sqlx::query_scalar("SELECT count(*) FROM organization_config.facility_session_policies WHERE facility_id=$1::text::numeric").bind(no_policy.value().to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(n, 0);
    sqlx::query("INSERT INTO organization_config.memberships(identity_ref,facility_id,version) VALUES($1::text::numeric,$2::text::numeric,1)")
        .bind((offset+2).to_string()).bind(a.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        membership::read_current(&mut c, identity(offset + 2), a)
            .await
            .unwrap()
            .is_none()
    );
    sqlx::query("INSERT INTO organization_config.memberships VALUES($1::text::numeric,$2::text::numeric,1,true)")
        .bind(u128::MAX.to_string()).bind(b.value().to_string()).execute(&mut c).await.unwrap();
    assert_eq!(
        membership::read_current(&mut c, identity(u128::MAX), b)
            .await
            .unwrap()
            .unwrap()
            .identity(),
        identity(u128::MAX)
    );
    // Canonical decoding also accepts integral numeric values with a decimal scale.
    sqlx::query("INSERT INTO organization_config.memberships VALUES($1::text::numeric,$2::text::numeric,1.0,true)")
        .bind(format!("{}.0",offset+9)).bind(b.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        membership::read_current(&mut c, identity(offset + 9), b)
            .await
            .unwrap()
            .is_some()
    );
    reject(
        &mut c,
        "INSERT INTO organization_config.memberships VALUES(0,99999,1,true)",
        "23514",
    )
    .await;
    reject(
        &mut c,
        "INSERT INTO organization_config.memberships VALUES(99999,99999,1,true)",
        "23503",
    )
    .await;
    reject(
        &mut c,
        "UPDATE organization_config.memberships SET active=NOT active",
        "23514",
    )
    .await;
    reject(
        &mut c,
        "UPDATE organization_config.memberships SET version=0",
        "23514",
    )
    .await;
    reject(
        &mut c,
        "UPDATE organization_config.memberships SET identity_ref=identity_ref+1,version=version+1",
        "23514",
    )
    .await;
    reject(
        &mut c,
        "UPDATE organization_config.memberships SET facility_id=facility_id+1,version=version+1",
        "23514",
    )
    .await;
    // A pending real writer does not expose half of a multi-row change to a plain read.
    let mut writer = connect("postgres").await;
    let mut pending = writer.begin().await.unwrap();
    sqlx::query("UPDATE organization_config.memberships SET active=false,version=version+1 WHERE identity_ref=$1::text::numeric AND facility_id=$2::text::numeric")
        .bind(person.value().to_string()).bind(a.value().to_string()).execute(&mut *pending).await.unwrap();
    sqlx::query("UPDATE organization_config.facility_session_policies SET version=version+1,idle_seconds=300,absolute_seconds=7200,step_up_seconds=120 WHERE facility_id=$1::text::numeric")
        .bind(a.value().to_string()).execute(&mut *pending).await.unwrap();
    let mut reader = connect("postgres").await;
    let during = tokio::time::timeout(
        Duration::from_secs(3),
        membership::read_current(&mut reader, person, a),
    )
    .await
    .unwrap()
    .unwrap()
    .unwrap();
    assert_eq!(during.version(), 1);
    assert_eq!(during.policy().version().value(), 1);
    assert_eq!(during.policy().limits(), SessionPolicyLimits::default());
    pending.commit().await.unwrap();
    assert!(
        membership::read_current(&mut reader, person, a)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        membership::read_current(&mut c, person, a)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(initial.version(), 1); // Old data is a snapshot, never an authorization permit.
    assert!(
        membership::read_current(&mut reader, person, b)
            .await
            .unwrap()
            .is_some()
    );
    session_policy::write(
        &mut c,
        b,
        Some(initial.policy().version()),
        SessionPolicyLimits::new(300, 7200, 120).unwrap(),
    )
    .await
    .unwrap();
    let latest = membership::read_current(&mut reader, person, b)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.policy().version().value(), 2);
    assert_eq!(latest.policy().limits().idle_seconds(), 300);
    let mut outer = c.begin().await.unwrap();
    assert!(
        membership::read_current(&mut outer, person, b)
            .await
            .is_err()
    );
    outer.rollback().await.unwrap();
    let mut runtime = connect("kc_membership_runtime").await;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM organization_config.memberships")
        .fetch_one(&mut runtime)
        .await
        .unwrap();
    assert_eq!(visible, 0);
    assert!(
        membership::read_current(&mut runtime, person, b)
            .await
            .unwrap()
            .is_none()
    );
    reject(
        &mut runtime,
        "INSERT INTO organization_config.memberships VALUES(9999,9999,1,true)",
        "42501",
    )
    .await;
    reject(
        &mut runtime,
        "UPDATE organization_config.memberships SET active=false,version=version+1",
        "42501",
    )
    .await;
    reject(
        &mut runtime,
        "DELETE FROM organization_config.memberships",
        "42501",
    )
    .await;
    reject(
        &mut runtime,
        "TRUNCATE organization_config.memberships",
        "42501",
    )
    .await;
    let mut owner = connect("postgres").await;
    sql(&mut owner, "SET ROLE kc_org_owner;").await;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM organization_config.memberships")
        .fetch_one(&mut owner)
        .await
        .unwrap();
    assert_eq!(visible, 0);
    assert!(
        membership::read_current(&mut owner, person, b)
            .await
            .unwrap()
            .is_none()
    );
    let closed:bool=sqlx::query_scalar("SELECT relrowsecurity AND relforcerowsecurity AND pg_get_userbyid(relowner)='kc_org_owner' FROM pg_catalog.pg_class WHERE oid='organization_config.memberships'::regclass").fetch_one(&mut c).await.unwrap();
    assert!(closed);
    let cross_service:i64=sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_constraint c JOIN pg_catalog.pg_class r ON r.oid=c.confrelid JOIN pg_catalog.pg_namespace n ON n.oid=r.relnamespace WHERE c.conrelid='organization_config.memberships'::regclass AND c.contype='f' AND n.nspname<>'organization_config'").fetch_one(&mut c).await.unwrap();
    assert_eq!(cross_service, 0);
    let versions: Vec<i32> = sqlx::query_scalar(
        "SELECT version FROM organization_config.schema_versions ORDER BY version",
    )
    .fetch_all(&mut c)
    .await
    .unwrap();
    assert_eq!(versions, [1, 2, 3, 4]);
    let reader_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut reader)
        .await
        .unwrap();
    sqlx::query("SELECT pg_terminate_backend($1)")
        .bind(reader_pid)
        .execute(&mut c)
        .await
        .unwrap();
    let failed = membership::read_current(&mut reader, person, b)
        .await
        .unwrap_err();
    assert_eq!(
        format!("{failed:?}"),
        "membership data unavailable or invalid"
    );
}
