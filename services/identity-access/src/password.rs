//! Internal password factor only. No login, session issuance or authorization.
use crate::IdentityId;
use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
    password_hash::phc::PasswordHash,
};
use sqlx::{Connection, PgConnection, Row};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use zeroize::Zeroizing;

/// Intentionally indistinguishable failure, including storage and throttling errors.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PasswordError;
impl fmt::Debug for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("password check unavailable or unsuccessful")
    }
}
impl std::error::Error for PasswordError {}
impl From<sqlx::Error> for PasswordError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}

pub struct Password {
    bytes: Zeroizing<[u8; 1024]>,
    len: usize,
}
impl Password {
    pub fn new(value: &[u8]) -> Result<Self, PasswordError> {
        if value.is_empty() || value.len() > 1024 {
            return Err(PasswordError);
        }
        let mut bytes = Zeroizing::new([0; 1024]);
        bytes[..value.len()].copy_from_slice(value);
        Ok(Self {
            bytes,
            len: value.len(),
        })
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password([REDACTED])")
    }
}

struct SecretHash(Zeroizing<String>);
impl SecretHash {
    fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Profile {
    Current,
    Previous,
}
impl Profile {
    fn engine(self) -> Result<Argon2<'static>, PasswordError> {
        let (memory, iterations) = match self {
            Self::Current => (65_536, 3),
            Self::Previous => (19_456, 2),
        };
        let params = Params::new(memory, iterations, 1, Some(32)).map_err(|_| PasswordError)?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
    fn parse(value: &str) -> Option<Self> {
        if value.len() > 256 {
            return None;
        }
        let hash = PasswordHash::new(value).ok()?;
        if hash.algorithm.as_str() != "argon2id"
            || hash.version != Some(19)
            || hash.salt.as_ref()?.as_ref().len() != 16
            || hash.hash.as_ref()?.len() != 32
        {
            return None;
        }
        match hash.params.as_str() {
            "m=65536,t=3,p=1" => Some(Self::Current),
            "m=19456,t=2,p=1" => Some(Self::Previous),
            _ => None,
        }
    }
    fn hash(self, password: &[u8]) -> Result<SecretHash, PasswordError> {
        let mut salt = [0; 16];
        getrandom::fill(&mut salt).map_err(|_| PasswordError)?;
        self.engine()?
            .hash_password_with_salt(password, &salt)
            .map(|hash| SecretHash::new(hash.to_string()))
            .map_err(|_| PasswordError)
    }
}

static WORKING: AtomicBool = AtomicBool::new(false);
struct Permit;
impl Drop for Permit {
    fn drop(&mut self) {
        WORKING.store(false, Ordering::Release);
    }
}
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, PasswordError> + Send + 'static,
) -> Result<T, PasswordError> {
    WORKING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .map_err(|_| PasswordError)?;
    let permit = Permit;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await
    .map_err(|_| PasswordError)?
}

struct DummyHashes {
    current: SecretHash,
    previous: SecretHash,
}
/// Reusable process-local comparison material. A process-wide permit bounds all instances.
pub struct PasswordChecker {
    dummy: Arc<DummyHashes>,
}
impl PasswordChecker {
    pub async fn new() -> Result<Self, PasswordError> {
        let dummy = blocking(|| {
            let mut random = Zeroizing::new([0; 32]);
            getrandom::fill(random.as_mut()).map_err(|_| PasswordError)?;
            Ok(DummyHashes {
                current: Profile::Current.hash(random.as_ref())?,
                previous: Profile::Previous.hash(random.as_ref())?,
            })
        })
        .await?;
        Ok(Self {
            dummy: Arc::new(dummy),
        })
    }

    /// Internal maintenance operation: caller must establish the authorized purpose first.
    /// Replacing a password revokes this identity's existing sessions in the same transaction.
    pub async fn store(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        password: Password,
    ) -> Result<(), PasswordError> {
        autocommit(connection)?;
        let hash = blocking(move || Profile::Current.hash(password.bytes())).await?;
        let revision = revision()?;
        let mut transaction = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result = sqlx::query("INSERT INTO identity_access.password_credentials (identity_id, phc, revision) VALUES ($1::text::numeric, $2, $3) ON CONFLICT (identity_id) DO UPDATE SET phc = EXCLUDED.phc, revision = EXCLUDED.revision")
            .bind(identity.value().to_string()).bind(hash.0.as_str()).bind(revision.as_slice()).execute(&mut *transaction).await?;
        if result.rows_affected() != 1 {
            return Err(PasswordError);
        }
        sqlx::query("UPDATE identity_access.sessions SET state = 'revoked' WHERE identity_id = $1::text::numeric AND state <> 'revoked'")
            .bind(identity.value().to_string()).execute(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(())
    }

    /// Always reserve first, including unknown accounts. Evidence is only a password factor.
    pub async fn check(
        &self,
        connection: &mut PgConnection,
        key: AttemptKey,
        limits: AttemptLimits,
        identity: Option<IdentityId>,
        password: Password,
    ) -> Result<PasswordEvidence, PasswordError> {
        reserve(connection, key, limits).await?;
        let snapshot = if let Some(identity) = identity {
            sqlx::query("SELECT phc, revision FROM identity_access.password_credentials WHERE identity_id = $1::text::numeric")
                .bind(identity.value().to_string()).fetch_optional(&mut *connection).await?
                .map(|row| -> Result<_, PasswordError> { Ok((SecretHash::new(row.try_get("phc")?), row.try_get::<Vec<u8>, _>("revision")?)) }).transpose()?
        } else {
            None
        };
        let dummy = Arc::clone(&self.dummy);
        let (verified, upgrade, snapshot) = blocking(move || {
            let profile = snapshot
                .as_ref()
                .and_then(|(hash, _)| Profile::parse(&hash.0));
            let current = if profile == Some(Profile::Current) {
                &snapshot.as_ref().ok_or(PasswordError)?.0
            } else {
                &dummy.current
            };
            let previous = if profile == Some(Profile::Previous) {
                &snapshot.as_ref().ok_or(PasswordError)?.0
            } else {
                &dummy.previous
            };
            let current_ok = Profile::Current
                .engine()?
                .verify_password(password.bytes(), current.0.as_str())
                .is_ok();
            let previous_ok = Profile::Previous
                .engine()?
                .verify_password(password.bytes(), previous.0.as_str())
                .is_ok();
            let verified = match profile {
                Some(Profile::Current) => current_ok,
                Some(Profile::Previous) => previous_ok,
                None => false,
            };
            let upgrade = if verified && profile == Some(Profile::Previous) {
                Some(Profile::Current.hash(password.bytes())?)
            } else {
                None
            };
            Ok((verified, upgrade, snapshot))
        })
        .await?;
        if !verified {
            return Err(PasswordError);
        }
        let identity = identity.ok_or(PasswordError)?;
        let (old_hash, old_revision) = snapshot.ok_or(PasswordError)?;
        let mut transaction = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let current: Option<Vec<u8>> = sqlx::query_scalar("SELECT revision FROM identity_access.password_credentials WHERE identity_id = $1::text::numeric AND phc = $2 FOR UPDATE")
            .bind(identity.value().to_string()).bind(old_hash.0.as_str()).fetch_optional(&mut *transaction).await?;
        if current != Some(old_revision) {
            return Err(PasswordError);
        }
        if let Some(hash) = upgrade {
            let result = sqlx::query("UPDATE identity_access.password_credentials SET phc = $2, revision = $3 WHERE identity_id = $1::text::numeric")
                .bind(identity.value().to_string()).bind(hash.0.as_str()).bind(revision()?.as_slice()).execute(&mut *transaction).await?;
            if result.rows_affected() != 1 {
                return Err(PasswordError);
            }
        }
        transaction.commit().await?;
        Ok(PasswordEvidence { identity })
    }
}

/// A local factor snapshot, never a session or permission. Pending MFA rules remain a release gate.
pub struct PasswordEvidence {
    identity: IdentityId,
}
impl PasswordEvidence {
    pub fn identity(&self) -> IdentityId {
        self.identity
    }
}
impl fmt::Debug for PasswordEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordEvidence([REDACTED])")
    }
}
fn revision() -> Result<[u8; 32], PasswordError> {
    let mut revision = [0; 32];
    getrandom::fill(&mut revision).map_err(|_| PasswordError)?;
    Ok(revision)
}
fn autocommit(connection: &PgConnection) -> Result<(), PasswordError> {
    if connection.is_in_transaction() {
        Err(PasswordError)
    } else {
        Ok(())
    }
}

/// Trusted service-internal pseudonymous key; never accept an unverified client-selected value.
#[derive(Clone, Copy)]
pub struct AttemptKey([u8; 32]);
impl AttemptKey {
    pub fn from_server_key(key: [u8; 32]) -> Self {
        Self(key)
    }
}

/// Explicit configuration, without production defaults or permanent account locking.
#[derive(Clone, Copy)]
pub struct AttemptLimits {
    window: u32,
    per_key: u32,
    total: u32,
    keys: u32,
}
impl AttemptLimits {
    pub fn new(
        window_seconds: u32,
        attempts_per_key: u32,
        total_attempts: u32,
        active_keys: u32,
    ) -> Result<Self, PasswordError> {
        if [
            window_seconds,
            attempts_per_key,
            total_attempts,
            active_keys,
        ]
        .contains(&0)
        {
            return Err(PasswordError);
        }
        Ok(Self {
            window: window_seconds,
            per_key: attempts_per_key,
            total: total_attempts,
            keys: active_keys,
        })
    }
}

async fn reserve(
    connection: &mut PgConnection,
    key: AttemptKey,
    limits: AttemptLimits,
) -> Result<(), PasswordError> {
    autocommit(connection)?;
    let mut transaction = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    sqlx::query("INSERT INTO identity_access.password_budget (id, window_seconds, per_key, total_limit, key_limit, window_start, last_observed, used) VALUES (1, $1, $2, $3, $4, 0, 0, 0) ON CONFLICT (id) DO NOTHING")
        .bind(i64::from(limits.window)).bind(i64::from(limits.per_key)).bind(i64::from(limits.total)).bind(i64::from(limits.keys)).execute(&mut *transaction).await?;
    let row = sqlx::query("SELECT window_seconds, per_key, total_limit, key_limit, window_start::text, last_observed::text, used FROM identity_access.password_budget WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *transaction).await?;
    for (name, expected) in [
        ("window_seconds", limits.window),
        ("per_key", limits.per_key),
        ("total_limit", limits.total),
        ("key_limit", limits.keys),
    ] {
        if row.try_get::<i64, _>(name)? != i64::from(expected) {
            return Err(PasswordError);
        }
    }
    let time: String = sqlx::query_scalar(
        "SELECT pg_catalog.floor(EXTRACT(epoch FROM pg_catalog.clock_timestamp()))::text",
    )
    .fetch_one(&mut *transaction)
    .await?;
    let time: u64 = time.parse().map_err(|_| PasswordError)?;
    let number = |name| -> Result<u64, PasswordError> {
        row.try_get::<String, _>(name)?
            .parse()
            .map_err(|_| PasswordError)
    };
    if time < number("last_observed")? {
        return Err(PasswordError);
    }
    let old_start = number("window_start")?;
    let (start, used) = if time
        >= old_start
            .checked_add(u64::from(limits.window))
            .ok_or(PasswordError)?
    {
        (time, 0)
    } else {
        (old_start, row.try_get::<i64, _>("used")?)
    };
    if used >= i64::from(limits.total) {
        return Err(PasswordError);
    }
    sqlx::query("DELETE FROM identity_access.password_attempts WHERE window_start + $1 <= $2::text::numeric")
        .bind(i64::from(limits.window)).bind(time.to_string()).execute(&mut *transaction).await?;
    let attempts: Option<i64> = sqlx::query_scalar(
        "SELECT used FROM identity_access.password_attempts WHERE subject_key = $1",
    )
    .bind(key.0.as_slice())
    .fetch_optional(&mut *transaction)
    .await?;
    if let Some(attempts) = attempts {
        if attempts >= i64::from(limits.per_key) {
            return Err(PasswordError);
        }
        let result = sqlx::query(
            "UPDATE identity_access.password_attempts SET used = used + 1 WHERE subject_key = $1",
        )
        .bind(key.0.as_slice())
        .execute(&mut *transaction)
        .await?;
        if result.rows_affected() != 1 {
            return Err(PasswordError);
        }
    } else {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM identity_access.password_attempts")
                .fetch_one(&mut *transaction)
                .await?;
        if count >= i64::from(limits.keys) {
            return Err(PasswordError);
        }
        let result = sqlx::query("INSERT INTO identity_access.password_attempts (subject_key, window_start, used) VALUES ($1, $2::text::numeric, 1)")
            .bind(key.0.as_slice()).bind(time.to_string()).execute(&mut *transaction).await?;
        if result.rows_affected() != 1 {
            return Err(PasswordError);
        }
    }
    let result = sqlx::query("UPDATE identity_access.password_budget SET window_start = $1::text::numeric, last_observed = $2::text::numeric, used = $3 WHERE id = 1")
        .bind(start.to_string()).bind(time.to_string()).bind(used + 1).execute(&mut *transaction).await?;
    if result.rows_affected() != 1 {
        return Err(PasswordError);
    }
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs_and_formats_are_bounded_without_hash_work() {
        assert_eq!(Password::new(&[]).unwrap_err(), PasswordError);
        assert_eq!(Password::new(&[7; 1025]).unwrap_err(), PasswordError);
        let password = Password::new(&[7; 1024]).unwrap();
        assert_eq!(password.bytes().len(), 1024);
        assert_eq!(format!("{password:?}"), "Password([REDACTED])");
        assert_eq!(format!("{PasswordError:?}"), PasswordError.to_string());
        assert!(AttemptLimits::new(0, 1, 1, 1).is_err());
        assert!(AttemptLimits::new(1, 0, 1, 1).is_err());
        assert!(AttemptLimits::new(1, 1, 0, 1).is_err());
        assert!(AttemptLimits::new(1, 1, 1, 0).is_err());
        for invalid in [
            "",
            "malformed",
            "$argon2id$v=19$m=4294967295,t=3,p=1$AA$AA",
            "$argon2i$v=19$m=65536,t=3,p=1$AA$AA",
        ] {
            assert!(Profile::parse(invalid).is_none());
        }
        assert!(Profile::parse(&"x".repeat(257)).is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancellation_holds_the_process_permit_until_blocking_work_finishes() {
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(blocking(move || {
            let _ = started_tx.send(());
            finish_rx.recv().map_err(|_| PasswordError)?;
            let _ = done_tx.send(());
            Ok(())
        }));
        started_rx.await.unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(blocking(|| Ok(())).await, Err(PasswordError));
        finish_tx.send(()).unwrap();
        done_rx.await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while WORKING.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(blocking(|| Ok(())).await, Ok(()));
    }
}
