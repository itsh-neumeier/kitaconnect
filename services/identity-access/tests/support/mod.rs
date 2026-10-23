use sqlx::PgConnection;
use std::time::Duration;

// Never use the failed connection here: doing so would flush a queued rollback.
pub async fn assert_idle(witness: &mut PgConnection, target: i32) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let idle: bool = sqlx::query_scalar("SELECT state = 'idle' AND xact_start IS NULL FROM pg_catalog.pg_stat_activity WHERE pid = $1")
                .bind(target).fetch_one(&mut *witness).await.unwrap();
            if idle {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("failed operation must finish its server transaction before returning");
}
