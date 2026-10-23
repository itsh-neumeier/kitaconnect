use kitaconnect_organization_config::{
    FacilityId,
    session_policy::{self, PolicyVersion, SessionPolicyLimits},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;

fn facility(value: u128) -> FacilityId {
    FacilityId::new(value).unwrap()
}
fn limits(idle: u64, absolute: u64, step_up: u64) -> SessionPolicyLimits {
    SessionPolicyLimits::new(idle, absolute, step_up).unwrap()
}
#[test]
fn approved_bounds_and_defaults_are_enforced() {
    let defaults = SessionPolicyLimits::default();
    assert_eq!(
        (
            defaults.idle_seconds(),
            defaults.absolute_seconds(),
            defaults.step_up_seconds()
        ),
        (900, 28_800, 300)
    );
    assert_eq!(limits(900, 28_800, 300), defaults);
    assert!(SessionPolicyLimits::new(1, 1, 1).is_ok());
    for (idle, absolute, step_up) in [
        (0, 100, 1),
        (1, 0, 1),
        (1, 100, 0),
        (901, 28_800, 1),
        (1, 28_801, 1),
        (1, 28_800, 301),
        (2, 1, 1),
        (1, 1, 2),
        (u64::MAX, u64::MAX, u64::MAX),
    ] {
        assert!(SessionPolicyLimits::new(idle, absolute, step_up).is_err());
    }
    assert!(PolicyVersion::new(0).is_none());
    assert_eq!(PolicyVersion::new(u64::MAX).unwrap().value(), u64::MAX);
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
            .username(user)
            .database("kc_session_policy_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn wait(c: &mut PgConnection, pid: i32) {
    for _ in 0..200 {
        let waiting:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid = $1 AND wait_event_type = 'Lock')").bind(pid).fetch_one(&mut *c).await.unwrap();
        if waiting {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("policy contender did not reach its row lock");
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn facility_session_policies_are_versioned_and_restricted() {
    let mut c = connect("postgres").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        let old = session_policy::read(&mut c, facility(1))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(old.version().value(), 3);
        assert!([limits(240, 6000, 100), limits(200, 5000, 90)].contains(&old.limits()));
        let second = session_policy::read(&mut c, facility(2))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second.version().value(), 1);
        assert_eq!(second.limits(), SessionPolicyLimits::default());
        assert_eq!(
            session_policy::read(&mut c, facility(3))
                .await
                .unwrap()
                .unwrap()
                .version()
                .value(),
            u64::MAX
        );
    }
    let offset = if after { 100 } else { 0 };
    let a = facility(offset + 1);
    let b = facility(offset + 2);
    let exhausted = facility(offset + 3);
    let empty = facility(offset + 4);
    for id in [a, b, exhausted, empty] {
        sqlx::query("INSERT INTO organization_config.facilities (id) VALUES ($1::text::numeric)")
            .bind(id.value().to_string())
            .execute(&mut c)
            .await
            .unwrap();
    }
    let defaults = SessionPolicyLimits::default();
    assert!(session_policy::read(&mut c, empty).await.unwrap().is_none());
    let n:i64=sqlx::query_scalar("SELECT count(*) FROM organization_config.facility_session_policies WHERE facility_id = $1::text::numeric")
        .bind(empty.value().to_string()).fetch_one(&mut c).await.unwrap();
    assert_eq!(n, 0); // Reading never creates defaults.
    assert!(
        session_policy::write(&mut c, facility(offset + 5), None, defaults)
            .await
            .is_err()
    );
    assert!(
        session_policy::write(&mut c, a, PolicyVersion::new(1), defaults)
            .await
            .is_err()
    );
    let original = session_policy::write(&mut c, a, None, defaults)
        .await
        .unwrap();
    assert_eq!(original.facility(), a);
    assert_eq!(original.version().value(), 1);
    assert!(
        session_policy::write(&mut c, a, None, defaults)
            .await
            .is_err()
    );
    let changed =
        session_policy::write(&mut c, a, Some(original.version()), limits(300, 7200, 120))
            .await
            .unwrap();
    assert_eq!(changed.version().value(), 2);
    assert!(
        session_policy::write(&mut c, a, Some(original.version()), defaults)
            .await
            .is_err()
    );
    session_policy::write(&mut c, b, None, defaults)
        .await
        .unwrap();
    assert_eq!(
        session_policy::read(&mut c, b)
            .await
            .unwrap()
            .unwrap()
            .limits(),
        defaults
    );
    assert!(
        session_policy::write(&mut c, b, Some(changed.version()), limits(100, 1000, 50))
            .await
            .is_err()
    );
    // Outer transactions are rejected before any update.
    let mut outer = c.begin().await.unwrap();
    assert!(
        session_policy::write(&mut outer, b, PolicyVersion::new(1), defaults)
            .await
            .is_err()
    );
    assert!(session_policy::read(&mut outer, b).await.is_err());
    outer.rollback().await.unwrap();
    // Two real row-lock waiters compare the same version; only one may succeed.
    let mut guard = connect("postgres").await;
    let mut lock = guard.begin().await.unwrap();
    sqlx::query("SELECT FROM organization_config.facility_session_policies WHERE facility_id = $1::text::numeric FOR UPDATE").bind(a.value().to_string()).execute(&mut *lock).await.unwrap();
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let lp = pid(&mut left).await;
    let rp = pid(&mut right).await;
    let l = tokio::spawn(async move {
        session_policy::write(
            &mut left,
            a,
            Some(changed.version()),
            limits(240, 6000, 100),
        )
        .await
    });
    let r = tokio::spawn(async move {
        session_policy::write(
            &mut right,
            a,
            Some(changed.version()),
            limits(200, 5000, 90),
        )
        .await
    });
    wait(&mut c, lp).await;
    wait(&mut c, rp).await;
    lock.commit().await.unwrap();
    assert_eq!(
        usize::from(l.await.unwrap().is_ok()) + usize::from(r.await.unwrap().is_ok()),
        1
    );
    assert_eq!(
        session_policy::read(&mut c, a)
            .await
            .unwrap()
            .unwrap()
            .version()
            .value(),
        3
    );
    // Suppressed writes and deferred commit failures cannot return success or change values.
    sql(&mut c,"CREATE FUNCTION public.kc_policy_suppress() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER kc_policy_suppress BEFORE UPDATE ON organization_config.facility_session_policies FOR EACH ROW EXECUTE FUNCTION public.kc_policy_suppress();").await;
    assert!(
        session_policy::write(&mut c, b, PolicyVersion::new(1), limits(100, 1000, 50))
            .await
            .is_err()
    );
    sql(&mut c,"DROP TRIGGER kc_policy_suppress ON organization_config.facility_session_policies; DROP FUNCTION public.kc_policy_suppress();").await;
    sql(&mut c,"CREATE FUNCTION public.kc_policy_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture commit failure' USING ERRCODE = '23514'; END $$; CREATE CONSTRAINT TRIGGER kc_policy_fail AFTER UPDATE ON organization_config.facility_session_policies DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.kc_policy_fail();").await;
    assert!(
        session_policy::write(&mut c, b, PolicyVersion::new(1), limits(100, 1000, 50))
            .await
            .is_err()
    );
    sql(&mut c,"DROP TRIGGER kc_policy_fail ON organization_config.facility_session_policies; DROP FUNCTION public.kc_policy_fail();").await;
    let unchanged = session_policy::read(&mut c, b).await.unwrap().unwrap();
    assert_eq!(unchanged.version().value(), 1);
    assert_eq!(unchanged.limits(), defaults);
    // Direct malformed SQL must also fail under the database constraints/trigger.
    for statement in [
        "UPDATE organization_config.facility_session_policies SET version = version WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET version = version+1, idle_seconds = 901 WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET version = version+1, absolute_seconds = 28801 WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET version = version+1, step_up_seconds = 301 WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET version = version+1, idle_seconds = 0 WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET version = version+1, idle_seconds = 1.5 WHERE facility_id = $1::text::numeric",
        "UPDATE organization_config.facility_session_policies SET facility_id = facility_id+1000, version = version+1 WHERE facility_id = $1::text::numeric",
    ] {
        assert!(
            sqlx::query(statement)
                .bind(b.value().to_string())
                .execute(&mut c)
                .await
                .is_err()
        );
    }
    // Fixture-only administrator setup for the overflow boundary.
    sql(&mut c,"ALTER TABLE organization_config.facility_session_policies DISABLE TRIGGER session_policy_guard;").await;
    sqlx::query("INSERT INTO organization_config.facility_session_policies VALUES ($1::text::numeric,18446744073709551615,900,28800,300)")
        .bind(exhausted.value().to_string()).execute(&mut c).await.unwrap();
    sql(&mut c,"ALTER TABLE organization_config.facility_session_policies ENABLE TRIGGER session_policy_guard;").await;
    assert!(
        session_policy::write(&mut c, exhausted, PolicyVersion::new(u64::MAX), defaults)
            .await
            .is_err()
    );
    // The real runtime still has no allowing RLS policy.
    let mut runtime = connect("kc_org_adapter_runtime").await;
    assert!(
        session_policy::read(&mut runtime, a)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        session_policy::write(&mut runtime, empty, None, defaults)
            .await
            .is_err()
    );
    assert!(
        session_policy::write(&mut runtime, a, PolicyVersion::new(3), defaults)
            .await
            .is_err()
    );
    let permitted:bool=sqlx::query_scalar("SELECT has_table_privilege('kc_org_runtime','organization_config.facility_session_policies','DELETE') OR has_column_privilege('kc_org_runtime','organization_config.facility_session_policies','facility_id','UPDATE')")
        .fetch_one(&mut c).await.unwrap();
    assert!(!permitted);
    let mut tx = c.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE kc_org_owner")
        .execute(&mut *tx)
        .await
        .unwrap();
    let visible: i64 =
        sqlx::query_scalar("SELECT count(*) FROM organization_config.facility_session_policies")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(visible, 0);
    tx.rollback().await.unwrap();
    // Lost connection is an error, not a default configuration.
    let mut dead = connect("postgres").await;
    let dead_pid = pid(&mut dead).await;
    let stopped: bool = sqlx::query_scalar("SELECT pg_terminate_backend($1)")
        .bind(dead_pid)
        .fetch_one(&mut c)
        .await
        .unwrap();
    assert!(stopped);
    assert!(session_policy::read(&mut dead, a).await.is_err());
}
