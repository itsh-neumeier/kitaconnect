use kitaconnect_identity_access::login_name::{LoginName, ReadLimits, Reader};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};

fn limits() -> ReadLimits {
    ReadLimits {
        parallel: 1,
        connect_time: Duration::from_secs(1),
        database_time: Duration::from_secs(1),
        total_time: Duration::from_secs(2),
    }
}
fn options(user: &str) -> PgConnectOptions {
    PgConnectOptions::new()
        .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated test socket required"))
        .username(user)
        .database("kc_login_name_test")
        .ssl_mode(PgSslMode::Disable)
}
fn reader(user: &str) -> Reader {
    Reader::new(options(user), limits()).unwrap()
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(&options(user)).await.unwrap()
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn resolved(r: &Reader, name: &str) -> Option<u128> {
    r.resolve(&LoginName::new(name).unwrap())
        .await
        .unwrap()
        .map(|id| id.value())
}
async fn insert(c: &mut PgConnection, name: &str, identity: i32) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO identity_access.login_names VALUES ($1,$2::integer,'active')")
        .bind(name.as_bytes())
        .bind(identity)
        .execute(c)
        .await
        .map(|_| ())
}
async fn rejected(c: &mut PgConnection, text: &'static str, state: &str) {
    let error = sqlx::raw_sql(text).execute(c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some(state)
    );
}

#[test]
fn names_are_bounded_utf8_and_diagnostics_redact_the_input() {
    for name in ["", &"x".repeat(257), &"ä".repeat(129)] {
        assert!(LoginName::new(name).is_err());
    }
    for name in [" ", "MixedCase", "é", "e\u{301}", &"ä".repeat(128)] {
        assert_eq!(
            format!("{:?}", LoginName::new(name).unwrap()),
            "LoginName(REDACTED)"
        );
    }
}

#[test]
fn invalid_work_limits_cannot_open_a_reader() {
    let options = PgConnectOptions::new();
    for bad in [
        ReadLimits {
            parallel: 0,
            ..limits()
        },
        ReadLimits {
            parallel: usize::MAX,
            ..limits()
        },
        ReadLimits {
            connect_time: Duration::ZERO,
            ..limits()
        },
        ReadLimits {
            database_time: Duration::ZERO,
            ..limits()
        },
        ReadLimits {
            total_time: Duration::ZERO,
            ..limits()
        },
        ReadLimits {
            connect_time: Duration::from_secs(3),
            ..limits()
        },
        ReadLimits {
            database_time: Duration::MAX,
            ..limits()
        },
        ReadLimits {
            total_time: Duration::MAX,
            ..limits()
        },
    ] {
        assert!(Reader::new(options.clone(), bad).is_err());
    }
}

async fn waiting(root: &mut PgConnection, expected: i64) {
    tokio::time::timeout(Duration::from_secs(5),async {
        loop {
            let n:i64=sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname='kc_login_name_test' AND application_name='kc_login_name_request' AND wait_event_type='Lock' AND state='active'")
                .fetch_one(&mut *root).await.unwrap();
            if n==expected {break;}
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("independent database wait witness");
}

#[tokio::test]
#[ignore = "requires the isolated PostgreSQL fixture"]
async fn exact_local_names_are_current_closed_and_persistent() {
    let mut root = connect("postgres").await;
    let r = reader("postgres");
    let runtime = reader("kc_login_name_runtime");
    if std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after" {
        assert_eq!(resolved(&r, "Name").await, None);
        assert_eq!(resolved(&r, "Race").await, Some(2));
        assert_eq!(resolved(&runtime, "Race").await, None);
        sql(&mut root, "TRUNCATE identity_access.login_names").await;
    } else {
        sql(
            &mut root,
            "INSERT INTO identity_access.identities SELECT generate_series(1,3)",
        )
        .await;
    }
    insert(&mut root, "Name", 1).await.unwrap();
    insert(&mut root, "é", 2).await.unwrap();
    assert_eq!(resolved(&r, "Name").await, Some(1));
    assert_eq!(resolved(&r, "é").await, Some(2));
    for name in ["name", " Name", "Name ", "unknown", "e\u{301}"] {
        assert_eq!(resolved(&r, name).await, None);
    }
    assert_eq!(resolved(&runtime, "Name").await, None);
    sql(&mut root, "SET ROLE kc_identity_owner").await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.login_names")
        .fetch_one(&mut root)
        .await
        .unwrap();
    assert_eq!(count, 0);
    sql(&mut root, "RESET ROLE").await;
    for (query, state) in [
        (
            "UPDATE identity_access.login_names SET identity_id=3",
            "23514",
        ),
        (
            "UPDATE identity_access.login_names SET name=decode('61','hex')",
            "23514",
        ),
        ("DELETE FROM identity_access.login_names", "23514"),
        (
            "INSERT INTO identity_access.login_names VALUES (decode('','hex'),1,'active')",
            "23514",
        ),
        (
            "INSERT INTO identity_access.login_names VALUES (decode(repeat('61',257),'hex'),1,'active')",
            "23514",
        ),
        (
            "INSERT INTO identity_access.login_names VALUES (decode('62','hex'),999,'active')",
            "23503",
        ),
    ] {
        rejected(&mut root, query, state).await;
    }
    let duplicate = insert(&mut root, "Name", 2).await.unwrap_err();
    assert_eq!(
        duplicate.as_database_error().unwrap().code().as_deref(),
        Some("23505")
    );
    let mut rc = connect("kc_login_name_runtime").await;
    rejected(
        &mut rc,
        "UPDATE identity_access.login_names SET state='revoked'",
        "42501",
    )
    .await;
    rejected(
        &mut rc,
        "INSERT INTO identity_access.login_names VALUES (decode('63','hex'),1,'active')",
        "42501",
    )
    .await;
    rejected(&mut rc, "TRUNCATE identity_access.login_names", "42501").await;

    let mut writer = connect("postgres").await;
    sql(&mut root, "BEGIN ISOLATION LEVEL REPEATABLE READ").await;
    let _: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.login_names")
        .fetch_one(&mut root)
        .await
        .unwrap();
    sql(
        &mut writer,
        "BEGIN; UPDATE identity_access.login_names SET state='revoked'",
    )
    .await;
    assert_eq!(resolved(&r, "Name").await, Some(1));
    sql(&mut writer, "ROLLBACK").await;
    assert_eq!(resolved(&r, "Name").await, Some(1));
    sql(
        &mut writer,
        "UPDATE identity_access.login_names SET state='revoked'",
    )
    .await;
    assert_eq!(resolved(&r, "Name").await, None);
    let frozen: String =
        sqlx::query_scalar("SELECT state FROM identity_access.login_names WHERE name=$1")
            .bind(b"Name".as_slice())
            .fetch_one(&mut root)
            .await
            .unwrap();
    assert_eq!(frozen, "active");
    sql(&mut root, "ROLLBACK").await;
    rejected(
        &mut root,
        "UPDATE identity_access.login_names SET state='active'",
        "23514",
    )
    .await;

    // Real unique-index contention, observed before the winner commits.
    sql(&mut writer, "BEGIN").await;
    insert(&mut writer, "Race", 2).await.unwrap();
    let mut contender = connect("postgres").await;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut contender)
        .await
        .unwrap();
    let task = tokio::spawn(async move { insert(&mut contender, "Race", 3).await });
    tokio::time::timeout(Duration::from_secs(5),async {
        loop {
            let wait:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock' AND state='active')").bind(pid).fetch_one(&mut root).await.unwrap();
            if wait {break;}
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    sql(&mut writer, "COMMIT").await;
    let error = tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23505")
    );
    assert_eq!(resolved(&r, "Race").await, Some(2));

    sql(&mut root,"CREATE POLICY synthetic_lookup ON identity_access.login_names FOR SELECT TO kc_identity_runtime USING (true)").await;
    assert_eq!(resolved(&runtime, "Race").await, Some(2));
    sql(
        &mut root,
        "REVOKE SELECT ON identity_access.login_names FROM kc_identity_runtime",
    )
    .await;
    let error = runtime
        .resolve(&LoginName::new("Race").unwrap())
        .await
        .unwrap_err();
    assert_eq!(format!("{error:?}"), "local account lookup unavailable");
    sql(&mut root,"GRANT SELECT ON identity_access.login_names TO kc_identity_runtime; DROP POLICY synthetic_lookup ON identity_access.login_names").await;
    assert_eq!(resolved(&runtime, "Race").await, None);
    let broken = Reader::new(
        options("postgres").database("kc_absent_login_name_database"),
        limits(),
    )
    .unwrap();
    assert!(
        broken
            .resolve(&LoginName::new("Race").unwrap())
            .await
            .is_err()
    );

    // Hold the blocker while the independent witness proves timeout/abort cleanup.
    sql(
        &mut writer,
        "BEGIN; LOCK TABLE identity_access.login_names IN ACCESS EXCLUSIVE MODE",
    )
    .await;
    let shared = Arc::new(reader("postgres"));
    assert!(
        tokio::time::timeout(
            Duration::from_secs(3),
            shared.resolve(&LoginName::new("Race").unwrap())
        )
        .await
        .unwrap()
        .is_err()
    );
    waiting(&mut root, 0).await;
    let clone = shared.clone();
    let task = tokio::spawn(async move { clone.resolve(&LoginName::new("Race").unwrap()).await });
    waiting(&mut root, 1).await;
    assert!(
        shared
            .resolve(&LoginName::new("Race").unwrap())
            .await
            .is_err()
    );
    waiting(&mut root, 1).await;
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    waiting(&mut root, 0).await;
    sql(&mut writer, "ROLLBACK").await;
    assert_eq!(resolved(&shared, "Race").await, Some(2));
}
