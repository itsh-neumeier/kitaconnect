//! Existing internal connection only. No role switch, connection creation or user authorization.
use crate::{
    IdentityId, SessionError, SessionLimits, SessionRecord, SessionRecordId, SessionState,
    token::SessionToken,
};
use sqlx::{Connection, PgConnection, Row};
use std::fmt;

pub enum StorageError {
    Database(sqlx::Error),
    Random(getrandom::Error),
    Model(SessionError),
    InvalidStoredRecord,
    TransactionActive,
    WriteNotConfirmed,
}

// Database internals and bearer material must not escape through normal diagnostics.
impl fmt::Debug for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("session storage operation failed")
    }
}
impl std::error::Error for StorageError {}
impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}
impl From<SessionError> for StorageError {
    fn from(value: SessionError) -> Self {
        Self::Model(value)
    }
}

fn require_autocommit(connection: &PgConnection) -> Result<(), StorageError> {
    if connection.is_in_transaction() {
        return Err(StorageError::TransactionActive);
    }
    Ok(())
}

async fn now(connection: &mut PgConnection) -> Result<u64, StorageError> {
    let value: String = sqlx::query_scalar(
        "SELECT pg_catalog.floor(EXTRACT(epoch FROM pg_catalog.clock_timestamp()))::text",
    )
    .fetch_one(connection)
    .await?;
    value.parse().map_err(|_| StorageError::InvalidStoredRecord)
}

/// Registers an internal identity key, without proving its owner or granting access.
pub async fn register_identity(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<(), StorageError> {
    require_autocommit(connection)?;
    let result =
        sqlx::query("INSERT INTO identity_access.identities (id) VALUES ($1::text::numeric)")
            .bind(identity.value().to_string())
            .execute(connection)
            .await?;
    if result.rows_affected() != 1 {
        return Err(StorageError::WriteNotConfirmed);
    }
    Ok(())
}

/// Internal caller must verify the complete authentication purpose before creating a session.
pub async fn create_session(
    connection: &mut PgConnection,
    id: SessionRecordId,
    identity: IdentityId,
    limits: SessionLimits,
) -> Result<SessionToken, StorageError> {
    require_autocommit(connection)?;
    let token = SessionToken::generate().map_err(StorageError::Random)?;
    let mut transaction = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    let created = now(&mut transaction).await?;
    let record = SessionRecord::new(id, identity, created, limits)?;
    let result = sqlx::query("INSERT INTO identity_access.sessions (id, identity_id, fingerprint, created_at, absolute_expires_at, idle_seconds, last_activity_at, last_observed_at, state) \
        VALUES ($1::text::numeric, $2::text::numeric, $3, $4::text::numeric, $5::text::numeric, $6::text::numeric, $4::text::numeric, $4::text::numeric, 'active')")
        .bind(id.value().to_string()).bind(identity.value().to_string()).bind(token.fingerprint().as_slice())
        .bind(created.to_string()).bind(record.absolute_expires_at().to_string()).bind(limits.idle_seconds.to_string())
        .execute(&mut *transaction).await?;
    if result.rows_affected() != 1 {
        return Err(StorageError::WriteNotConfirmed);
    }
    transaction.commit().await?;
    Ok(token)
}

fn decode(row: &sqlx::postgres::PgRow) -> Result<SessionRecord, StorageError> {
    fn number<T: std::str::FromStr>(
        row: &sqlx::postgres::PgRow,
        column: &str,
    ) -> Result<T, StorageError> {
        row.try_get::<String, _>(column)?
            .parse()
            .map_err(|_| StorageError::InvalidStoredRecord)
    }
    let id = SessionRecordId::new(number(row, "id")?).ok_or(StorageError::InvalidStoredRecord)?;
    let identity =
        IdentityId::new(number(row, "identity_id")?).ok_or(StorageError::InvalidStoredRecord)?;
    let created_at = number(row, "created_at")?;
    let absolute_expires_at: u64 = number(row, "absolute_expires_at")?;
    let absolute = absolute_expires_at
        .checked_sub(created_at)
        .ok_or(StorageError::InvalidStoredRecord)?;
    let limits = SessionLimits::new(number(row, "idle_seconds")?, absolute)?;
    let mut record = SessionRecord::new(id, identity, created_at, limits)?;
    record.last_activity_at = number(row, "last_activity_at")?;
    record.last_observed_at = number(row, "last_observed_at")?;
    record.state = match row.try_get::<&str, _>("state")? {
        "active" => SessionState::Active,
        "expired" => SessionState::Expired,
        "revoked" => SessionState::Revoked,
        _ => return Err(StorageError::InvalidStoredRecord),
    };
    if record.last_activity_at < created_at
        || record.last_activity_at >= absolute_expires_at
        || record.last_observed_at < record.last_activity_at
        || (record.state == SessionState::Active && record.last_observed_at >= record.expires_at())
    {
        return Err(StorageError::InvalidStoredRecord);
    }
    Ok(record)
}

async fn current(
    connection: &mut PgConnection,
    token: &SessionToken,
    activity: bool,
) -> Result<Option<SessionRecord>, StorageError> {
    require_autocommit(connection)?;
    let mut transaction = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    let row = sqlx::query("SELECT pg_catalog.trim_scale(id)::text AS id, pg_catalog.trim_scale(identity_id)::text AS identity_id, \
        pg_catalog.trim_scale(created_at)::text AS created_at, pg_catalog.trim_scale(absolute_expires_at)::text AS absolute_expires_at, \
        pg_catalog.trim_scale(idle_seconds)::text AS idle_seconds, pg_catalog.trim_scale(last_activity_at)::text AS last_activity_at, \
        pg_catalog.trim_scale(last_observed_at)::text AS last_observed_at, state \
        FROM identity_access.sessions WHERE fingerprint = $1 FOR UPDATE")
        .bind(token.fingerprint().as_slice()).fetch_optional(&mut *transaction).await?;
    let Some(row) = row else {
        transaction.commit().await?;
        return Ok(None);
    };
    let mut record = decode(&row)?;
    // Read after FOR UPDATE, not transaction start: a blocked query may cross an expiry boundary.
    let time = now(&mut transaction).await?;
    let state = record.observe(time)?;
    if activity && state == SessionState::Active {
        record.record_activity(time)?;
    }
    let state_name = match record.state {
        SessionState::Active => "active",
        SessionState::Expired => "expired",
        SessionState::Revoked => "revoked",
    };
    let changed = sqlx::query("UPDATE identity_access.sessions SET last_activity_at = $2::text::numeric, last_observed_at = $3::text::numeric, state = $4 WHERE id = $1::text::numeric")
        .bind(record.id.value().to_string()).bind(record.last_activity_at.to_string())
        .bind(record.last_observed_at.to_string()).bind(state_name).execute(&mut *transaction).await?.rows_affected();
    if changed != 1 {
        return Err(StorageError::WriteNotConfirmed);
    }
    transaction.commit().await?;
    Ok(if state == SessionState::Active {
        Some(record)
    } else {
        None
    })
}

/// Current local snapshot only; every later protected action must read again.
pub async fn inspect(
    connection: &mut PgConnection,
    token: &SessionToken,
) -> Result<Option<SessionRecord>, StorageError> {
    current(connection, token, false).await
}

/// Only for currently authorized activity, never arbitrary client polling.
pub async fn record_activity(
    connection: &mut PgConnection,
    token: &SessionToken,
) -> Result<Option<SessionRecord>, StorageError> {
    current(connection, token, true).await
}

/// Internal idempotent revocation; caller authorization is outside this storage operation.
pub async fn revoke(
    connection: &mut PgConnection,
    id: SessionRecordId,
) -> Result<bool, StorageError> {
    require_autocommit(connection)?;
    Ok(sqlx::query(
        "UPDATE identity_access.sessions SET state = 'revoked' WHERE id = $1::text::numeric",
    )
    .bind(id.value().to_string())
    .execute(connection)
    .await?
    .rows_affected()
        != 0)
}
