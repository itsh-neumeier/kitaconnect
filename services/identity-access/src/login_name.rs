//! Exact internal account lookup only. Never a factor, membership or session proof.
use crate::IdentityId;
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};
use std::{
    fmt,
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, time::timeout};
use zeroize::Zeroizing;

#[derive(Clone, Copy)]
pub struct ReadError;
impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("local account lookup unavailable")
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for ReadError {}

/// Exact UTF-8 bytes, not a public naming policy or authentication proof.
pub struct LoginName(Zeroizing<String>);
impl LoginName {
    pub fn new(value: &str) -> Result<Self, ReadError> {
        if value.is_empty() || value.len() > 256 {
            return Err(ReadError);
        }
        Ok(Self(Zeroizing::new(value.to_owned())))
    }
}
impl fmt::Debug for LoginName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginName(REDACTED)")
    }
}

#[derive(Clone, Copy)]
pub struct ReadLimits {
    pub parallel: usize,
    pub connect_time: Duration,
    pub database_time: Duration,
    pub total_time: Duration,
}
pub struct Reader {
    database: PgConnectOptions,
    limits: ReadLimits,
    budget: Semaphore,
}
impl Reader {
    /// Every read owns a fresh connection; no caller transaction or positive cache.
    pub fn new(database: PgConnectOptions, limits: ReadLimits) -> Result<Self, ReadError> {
        let ms = limits.database_time.as_nanos().div_ceil(1_000_000);
        if limits.parallel == 0
            || limits.parallel > Semaphore::MAX_PERMITS
            || limits.connect_time > limits.total_time
            || ms == 0
            || ms > i32::MAX as u128
            || [limits.connect_time, limits.database_time, limits.total_time]
                .iter()
                .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(ReadError);
        }
        let ms = ms.to_string();
        let database = database.application_name("kc_login_name_request").options([
            ("statement_timeout", ms.as_str()),
            ("transaction_timeout", ms.as_str()),
            ("idle_in_transaction_session_timeout", ms.as_str()),
        ]);
        Ok(Self {
            database,
            limits,
            budget: Semaphore::new(limits.parallel),
        })
    }
    /// None means no visible active assignment. It must never become a public existence oracle.
    pub async fn resolve(&self, name: &LoginName) -> Result<Option<IdentityId>, ReadError> {
        let _permit = self.budget.try_acquire().map_err(|_| ReadError)?;
        timeout(self.limits.total_time, async {
            let mut c = timeout(self.limits.connect_time, PgConnection::connect_with(&self.database))
                .await.map_err(|_| ReadError)?.map_err(|_| ReadError)?;
            let value: Option<String> = sqlx::query_scalar(
                "SELECT identity_id::text FROM identity_access.login_names WHERE name=$1 AND state='active'")
                .bind(name.0.as_bytes()).fetch_optional(&mut c).await.map_err(|_| ReadError)?;
            value.map(|v| v.parse().ok().and_then(IdentityId::new).ok_or(ReadError)).transpose()
        }).await.map_err(|_| ReadError)?
    }
}
