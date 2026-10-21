//! Confirmed localhost test profile only. No HTTP enrollment, session issuance or permissions.
use crate::{IdentityId, token::SessionToken};
use sqlx::{Connection, PgConnection, Row};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use webauthn_rs::prelude::*;

const MAX_RESPONSE: usize = 65_536;
const PROFILE: &str = "localhost:3000/v1";
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WebauthnError;
impl fmt::Display for WebauthnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("passkey check unavailable or unsuccessful")
    }
}
impl fmt::Debug for WebauthnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for WebauthnError {}
impl From<sqlx::Error> for WebauthnError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}
impl From<serde_json::Error> for WebauthnError {
    fn from(_: serde_json::Error) -> Self {
        Self
    }
}

/// Distinct from a session token, credential ID and library challenge.
pub struct CeremonyToken(SessionToken);
impl CeremonyToken {
    pub fn from_bytes(value: &[u8]) -> Option<Self> {
        SessionToken::from_bytes(value).map(Self)
    }
    /// Explicit transport only: never log or persist raw reference material.
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose()
    }
}
impl fmt::Debug for CeremonyToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CeremonyToken([REDACTED])")
    }
}
pub struct CredentialRef(Vec<u8>);
impl CredentialRef {
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}
impl fmt::Debug for CredentialRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CredentialRef([REDACTED])")
    }
}
pub struct PasskeyEvidence(IdentityId);
impl PasskeyEvidence {
    pub fn identity(&self) -> IdentityId {
        self.0
    }
}
impl fmt::Debug for PasskeyEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasskeyEvidence([REDACTED])")
    }
}

/// Explicit synthetic-use limits, not production policy defaults.
#[derive(Clone, Copy)]
pub struct CeremonyLimits {
    seconds: u32,
    pending: u32,
    credentials: u32,
}
impl CeremonyLimits {
    pub fn new(
        lifetime_seconds: u32,
        stored_ceremonies: u32,
        credentials_per_identity: u32,
    ) -> Result<Self, WebauthnError> {
        if lifetime_seconds == 0
            || lifetime_seconds.checked_mul(1000).is_none()
            || stored_ceremonies == 0
            || !(1..=32).contains(&credentials_per_identity)
        {
            return Err(WebauthnError);
        }
        Ok(Self {
            seconds: lifetime_seconds,
            pending: stored_ceremonies,
            credentials: credentials_per_identity,
        })
    }
}

pub struct LocalhostWebauthn {
    engine: Arc<Webauthn>,
    limits: CeremonyLimits,
}
struct Account {
    handle: Uuid,
    revision: Vec<u8>,
    observed: u64,
}
struct Consumed {
    state: String,
    revision: Vec<u8>,
    created: u64,
    expires: u64,
}

fn random_revision() -> Result<[u8; 32], WebauthnError> {
    let mut value = [0; 32];
    getrandom::fill(&mut value).map_err(|_| WebauthnError)?;
    Ok(value)
}
fn autocommit(connection: &PgConnection) -> Result<(), WebauthnError> {
    if connection.is_in_transaction() {
        Err(WebauthnError)
    } else {
        Ok(())
    }
}
async fn now(connection: &mut PgConnection) -> Result<u64, WebauthnError> {
    let value: String = sqlx::query_scalar(
        "SELECT pg_catalog.floor(EXTRACT(epoch FROM pg_catalog.clock_timestamp()))::text",
    )
    .fetch_one(connection)
    .await?;
    value.parse().map_err(|_| WebauthnError)
}
fn number(row: &sqlx::postgres::PgRow, column: &str) -> Result<u64, WebauthnError> {
    row.try_get::<String, _>(column)?
        .parse()
        .map_err(|_| WebauthnError)
}
async fn account(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<Account, WebauthnError> {
    let row = sqlx::query("SELECT user_handle::text, revision, last_observed::text FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric FOR UPDATE")
        .bind(identity.value().to_string()).fetch_one(connection).await?;
    Ok(Account {
        handle: row
            .try_get::<String, _>("user_handle")?
            .parse()
            .map_err(|_| WebauthnError)?,
        revision: row.try_get("revision")?,
        observed: number(&row, "last_observed")?,
    })
}
async fn keys(
    connection: &mut PgConnection,
    identity: IdentityId,
    max: u32,
) -> Result<Vec<Passkey>, WebauthnError> {
    let rows = sqlx::query("SELECT credential_id, data FROM identity_access.webauthn_credentials WHERE identity_id = $1::text::numeric ORDER BY credential_id LIMIT 33")
        .bind(identity.value().to_string()).fetch_all(connection).await?;
    if rows.len() > max as usize {
        return Err(WebauthnError);
    }
    rows.into_iter()
        .map(|row| {
            let value: String = row.try_get("data")?;
            if value.len() > MAX_RESPONSE {
                return Err(WebauthnError);
            }
            let key: Passkey = serde_json::from_str(&value)?;
            let stored: Vec<u8> = row.try_get("credential_id")?;
            if key.cred_id().as_slice() != stored {
                return Err(WebauthnError);
            }
            Ok(key)
        })
        .collect()
}
async fn observe(
    connection: &mut PgConnection,
    identity: IdentityId,
    time: u64,
    revision: &[u8],
) -> Result<(), WebauthnError> {
    let result = sqlx::query("UPDATE identity_access.webauthn_accounts SET last_observed = $2::text::numeric, revision = $3 WHERE identity_id = $1::text::numeric")
        .bind(identity.value().to_string()).bind(time.to_string()).bind(revision).execute(connection).await?;
    if result.rows_affected() != 1 {
        return Err(WebauthnError);
    }
    Ok(())
}
static VERIFYING: AtomicBool = AtomicBool::new(false);
struct Permit;
impl Drop for Permit {
    fn drop(&mut self) {
        VERIFYING.store(false, Ordering::Release);
    }
}
async fn verify<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, WebauthnError> + Send + 'static,
) -> Result<T, WebauthnError> {
    VERIFYING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .map_err(|_| WebauthnError)?;
    let permit = Permit;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await
    .map_err(|_| WebauthnError)?
}

impl LocalhostWebauthn {
    /// No configurable production origin or automatic fallback exists in this test adapter.
    pub fn new(limits: CeremonyLimits) -> Result<Self, WebauthnError> {
        let origin = Url::parse("http://localhost:3000").map_err(|_| WebauthnError)?;
        let engine = WebauthnBuilder::new("localhost", &origin)
            .map_err(|_| WebauthnError)?
            .allow_subdomains(false)
            .allow_any_port(false)
            .timeout(Duration::from_secs(u64::from(limits.seconds)))
            .build()
            .map_err(|_| WebauthnError)?;
        Ok(Self {
            engine: Arc::new(engine),
            limits,
        })
    }

    async fn begin<'c>(
        &self,
        connection: &'c mut PgConnection,
        identity: IdentityId,
    ) -> Result<(sqlx::Transaction<'c, sqlx::Postgres>, Account, u64), WebauthnError> {
        autocommit(connection)?;
        // Lock order: start quota -> account; completion releases ceremony lock before account lock.
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result: Result<(Account, u64), WebauthnError> = async {
            crate::login::lock_identity(&mut tx, identity)
                .await
                .map_err(|_| WebauthnError)?;
            sqlx::query("SELECT pg_catalog.pg_advisory_xact_lock(184006, 3)")
                .execute(&mut *tx)
                .await?;
            let mut handle = [0; 16];
            getrandom::fill(&mut handle).map_err(|_| WebauthnError)?;
            sqlx::query("INSERT INTO identity_access.webauthn_accounts (identity_id, user_handle, revision, last_observed) VALUES ($1::text::numeric, $2::text::uuid, $3, 0) ON CONFLICT (identity_id) DO NOTHING")
                .bind(identity.value().to_string()).bind(Uuid::from_bytes(handle).to_string()).bind(random_revision()?.as_slice()).execute(&mut *tx).await?;
            let account = account(&mut tx, identity).await?;
            let time = now(&mut tx).await?;
            if time < account.observed {
                return Err(WebauthnError);
            }
            sqlx::query(
                "DELETE FROM identity_access.webauthn_ceremonies WHERE expires_at <= $1::text::numeric",
            )
            .bind(time.to_string())
            .execute(&mut *tx)
            .await?;
            let count: i64 =
                sqlx::query_scalar("SELECT count(*) FROM identity_access.webauthn_ceremonies")
                    .fetch_one(&mut *tx)
                    .await?;
            if count >= i64::from(self.limits.pending) {
                return Err(WebauthnError);
            }
            Ok((account, time))
        }.await;
        match result {
            Ok((account, time)) => Ok((tx, account, time)),
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    async fn save(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        account: &Account,
        time: u64,
        purpose: &str,
        state: String,
    ) -> Result<CeremonyToken, WebauthnError> {
        if state.len() > MAX_RESPONSE {
            return Err(WebauthnError);
        }
        let token = CeremonyToken(SessionToken::generate().map_err(|_| WebauthnError)?);
        let expires = time
            .checked_add(u64::from(self.limits.seconds))
            .ok_or(WebauthnError)?;
        let result = sqlx::query("INSERT INTO identity_access.webauthn_ceremonies (fingerprint, identity_id, profile, purpose, created_at, expires_at, consumed, expected_revision, state) VALUES ($1, $2::text::numeric, $3, $4, $5::text::numeric, $6::text::numeric, false, $7, $8)")
            .bind(token.0.fingerprint().as_slice()).bind(identity.value().to_string()).bind(PROFILE).bind(purpose).bind(time.to_string()).bind(expires.to_string()).bind(&account.revision).bind(state).execute(&mut *connection).await?;
        if result.rows_affected() != 1 {
            return Err(WebauthnError);
        }
        observe(connection, identity, time, &account.revision).await?;
        Ok(token)
    }

    /// Internal registration requires an already verified enrollment purpose. No password prerequisite.
    pub async fn start_registration(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
    ) -> Result<(CeremonyToken, CreationChallengeResponse), WebauthnError> {
        let (mut tx, account, time) = self.begin(connection, identity).await?;
        let result: Result<(CeremonyToken, CreationChallengeResponse), WebauthnError> = async {
            let keys = keys(&mut tx, identity, self.limits.credentials).await?;
            if keys.len() >= self.limits.credentials as usize {
                return Err(WebauthnError);
            }
            // Only an opaque handle is sent to the authenticator: no email, names or child data.
            let name = account.handle.to_string();
            let (options, state) = self
                .engine
                .start_passkey_registration(
                    account.handle,
                    &name,
                    &name,
                    Some(keys.iter().map(|key| key.cred_id().clone()).collect()),
                )
                .map_err(|_| WebauthnError)?;
            let token = self
                .save(
                    &mut tx,
                    identity,
                    &account,
                    time,
                    "register",
                    serde_json::to_string(&state)?,
                )
                .await?;

            Ok((token, options))
        }
        .await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
    pub async fn start_authentication(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
    ) -> Result<(CeremonyToken, RequestChallengeResponse), WebauthnError> {
        let (mut tx, account, time) = self.begin(connection, identity).await?;
        let result: Result<(CeremonyToken, RequestChallengeResponse), WebauthnError> = async {
            let keys = keys(&mut tx, identity, self.limits.credentials).await?;
            if keys.is_empty() {
                return Err(WebauthnError);
            }
            let (options, state) = self
                .engine
                .start_passkey_authentication(&keys)
                .map_err(|_| WebauthnError)?;
            let token = self
                .save(
                    &mut tx,
                    identity,
                    &account,
                    time,
                    "authenticate",
                    serde_json::to_string(&state)?,
                )
                .await?;

            Ok((token, options))
        }
        .await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    async fn consume(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        token: &CeremonyToken,
        purpose: &str,
    ) -> Result<Consumed, WebauthnError> {
        autocommit(connection)?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result: Result<_, WebauthnError> = async {
            let row = sqlx::query("UPDATE identity_access.webauthn_ceremonies SET consumed = true WHERE fingerprint = $1 AND NOT consumed RETURNING identity_id::text, profile, purpose, created_at::text, expires_at::text, expected_revision, state")
                .bind(token.0.fingerprint().as_slice()).fetch_optional(&mut *tx).await?.ok_or(WebauthnError)?;
            let time = now(&mut tx).await?; // Read after the winning update acquired its lock.
            Ok((row, time))
        }.await;
        let (row, time) = match result {
            Ok(value) => {
                tx.commit().await?; // Later validation or cancellation never restores the attempt.
                value
            }
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        };
        let created = number(&row, "created_at")?;
        let expires = number(&row, "expires_at")?;
        if row.try_get::<String, _>("identity_id")? != identity.value().to_string()
            || row.try_get::<String, _>("purpose")? != purpose
            || row.try_get::<String, _>("profile")? != PROFILE
            || time < created
            || time >= expires
        {
            return Err(WebauthnError);
        }
        Ok(Consumed {
            state: row.try_get("state")?,
            revision: row.try_get("expected_revision")?,
            created,
            expires,
        })
    }
    async fn current(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        consumed: &Consumed,
    ) -> Result<Account, WebauthnError> {
        let account = account(connection, identity).await?;
        if account.revision != consumed.revision {
            return Err(WebauthnError);
        }
        Ok(account)
    }
    async fn finish_time(
        connection: &mut PgConnection,
        account: &Account,
        consumed: &Consumed,
    ) -> Result<u64, WebauthnError> {
        let time = now(connection).await?;
        if time < account.observed || time < consumed.created || time >= consumed.expires {
            return Err(WebauthnError);
        }
        Ok(time)
    }

    pub async fn finish_registration(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        token: &CeremonyToken,
        response: &[u8],
    ) -> Result<CredentialRef, WebauthnError> {
        let consumed = self
            .consume(connection, identity, token, "register")
            .await?;
        if response.len() > MAX_RESPONSE || consumed.state.len() > MAX_RESPONSE {
            return Err(WebauthnError);
        }
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result: Result<CredentialRef, WebauthnError> = async {
            crate::login::lock_identity(&mut tx, identity)
                .await
                .map_err(|_| WebauthnError)?;
            let account = self.current(&mut tx, identity, &consumed).await?;
            Self::finish_time(&mut tx, &account, &consumed).await?;
            let state: PasskeyRegistration = serde_json::from_str(&consumed.state)?;
            let response: RegisterPublicKeyCredential = serde_json::from_slice(response)?;
            let engine = Arc::clone(&self.engine);
            let key = verify(move || {
                engine
                    .finish_passkey_registration(&response, &state)
                    .map_err(|_| WebauthnError)
            })
            .await?;
            let current = keys(&mut tx, identity, self.limits.credentials).await?;
            if current.len() >= self.limits.credentials as usize
                || !(1..=1024).contains(&key.cred_id().len())
            {
                return Err(WebauthnError);
            }
            let reference = CredentialRef(key.cred_id().as_slice().to_vec());
            let result = sqlx::query("INSERT INTO identity_access.webauthn_credentials (credential_id, identity_id, data) VALUES ($1, $2::text::numeric, $3)")
                .bind(reference.expose()).bind(identity.value().to_string()).bind(serde_json::to_string(&key)?).execute(&mut *tx).await?;
            if result.rows_affected() != 1 {
                return Err(WebauthnError);
            }
            let time = Self::finish_time(&mut tx, &account, &consumed).await?;
            observe(&mut tx, identity, time, &random_revision()?).await?;

            Ok(reference)
        }.await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
    pub async fn finish_authentication(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        token: &CeremonyToken,
        response: &[u8],
    ) -> Result<PasskeyEvidence, WebauthnError> {
        self.finish_authentication_inner(connection, identity, token, response, None)
            .await
    }
    pub async fn finish_authentication_for_login(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        token: &CeremonyToken,
        response: &[u8],
        flow: &crate::login::LoginToken,
    ) -> Result<(), WebauthnError> {
        self.finish_authentication_inner(connection, identity, token, response, Some(flow))
            .await
            .map(|_| ())
    }
    async fn finish_authentication_inner(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        token: &CeremonyToken,
        response: &[u8],
        flow: Option<&crate::login::LoginToken>,
    ) -> Result<PasskeyEvidence, WebauthnError> {
        let consumed = self
            .consume(connection, identity, token, "authenticate")
            .await?;
        if response.len() > MAX_RESPONSE || consumed.state.len() > MAX_RESPONSE {
            return Err(WebauthnError);
        }
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result: Result<Option<PasskeyEvidence>, WebauthnError> = async {
            crate::login::lock_identity(&mut tx, identity)
                .await
                .map_err(|_| WebauthnError)?;
            let account = self.current(&mut tx, identity, &consumed).await?;
            Self::finish_time(&mut tx, &account, &consumed).await?;
            let state: PasskeyAuthentication = serde_json::from_str(&consumed.state)?;
            let response: PublicKeyCredential = serde_json::from_slice(response)?;
            if response
                .get_user_unique_id()
                .is_some_and(|handle| handle != account.handle.as_bytes())
            {
                return Err(WebauthnError);
            }
            let engine = Arc::clone(&self.engine);
            let result = verify(move || {
                engine
                    .finish_passkey_authentication(&response, &state)
                    .map_err(|_| WebauthnError)
            })
            .await?;
            if !result.user_verified() {
                return Err(WebauthnError);
            }
            let mut keys = keys(&mut tx, identity, self.limits.credentials).await?;
            let key = keys
                .iter_mut()
                .find(|key| key.cred_id() == result.cred_id())
                .ok_or(WebauthnError)?;
            key.update_credential(&result).ok_or(WebauthnError)?;
            let written = sqlx::query("UPDATE identity_access.webauthn_credentials SET data = $2 WHERE credential_id = $1 AND identity_id = $3::text::numeric")
                .bind(key.cred_id().as_slice()).bind(serde_json::to_string(key)?).bind(identity.value().to_string()).execute(&mut *tx).await?;
            if written.rows_affected() != 1 {
                return Err(WebauthnError);
            }
            let time = Self::finish_time(&mut tx, &account, &consumed).await?;
            observe(&mut tx, identity, time, &random_revision()?).await?;
            if let Some(flow) = flow {
                let recorded =
                    crate::login::record_factor(&mut tx, identity, flow, crate::login::Factor::Passkey)
                        .await
                        .map_err(|_| WebauthnError)?;
                match recorded {
                    crate::login::FactorRecord::Recorded => {}
                    crate::login::FactorRecord::Expired => {
                        return Ok(None);
                    }
                }
            }

            Ok(Some(PasskeyEvidence(identity)))
        }.await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                value.ok_or(WebauthnError)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    /// Internal authenticated maintenance only. Invalidates all outstanding snapshots for this identity.
    pub async fn revoke(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        credential: &CredentialRef,
    ) -> Result<bool, WebauthnError> {
        autocommit(connection)?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        let result: Result<bool, WebauthnError> = async {
            crate::login::lock_identity(&mut tx, identity)
                .await
                .map_err(|_| WebauthnError)?;
            let account = account(&mut tx, identity).await?;
            let time = now(&mut tx).await?;
            if time < account.observed {
                return Err(WebauthnError);
            }
            let result = sqlx::query("DELETE FROM identity_access.webauthn_credentials WHERE identity_id = $1::text::numeric AND credential_id = $2")
                .bind(identity.value().to_string()).bind(credential.expose()).execute(&mut *tx).await?;
            observe(&mut tx, identity, time, &random_revision()?).await?;
            crate::login::revoke_sessions(&mut tx, identity)
                .await
                .map_err(|_| WebauthnError)?;

            Ok(result.rows_affected() == 1)
        }.await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
}
