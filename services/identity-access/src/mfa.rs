//! Internal synthetic-key MFA adapter. No HTTP authentication or production key loading.
use crate::{
    IdentityId,
    password::{self, AttemptKey, AttemptLimits},
    token::SessionToken,
};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, Row};
use std::{fmt, sync::Arc};
use totp_rs::{Algorithm, Builder, Secret};
use zeroize::Zeroizing;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MfaError;
impl fmt::Display for MfaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MFA operation unavailable or unsuccessful")
    }
}

impl fmt::Debug for MfaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for MfaError {}
impl From<sqlx::Error> for MfaError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}
fn random<const N: usize>() -> Result<[u8; N], MfaError> {
    let mut value = [0; N];
    getrandom::fill(&mut value).map_err(|_| MfaError)?;
    Ok(value)
}

pub struct EphemeralMfaKey {
    tag: [u8; 16],
    bytes: Zeroizing<[u8; 32]>,
}
impl EphemeralMfaKey {
    /// Fresh RAM key only. No import/export or production provider is implemented.
    pub fn generate() -> Result<Self, MfaError> {
        Ok(Self {
            tag: random()?,
            bytes: Zeroizing::new(random()?),
        })
    }
}
impl fmt::Debug for EphemeralMfaKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EphemeralMfaKey([REDACTED])")
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FactorId([u8; 16]);
impl fmt::Debug for FactorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FactorId([REDACTED])")
    }
}
pub struct TotpSecret(Zeroizing<[u8; 20]>);
impl TotpSecret {
    /// One protected enrollment boundary. Never log these bytes or the resulting library object.
    pub fn expose(&self) -> &[u8; 20] {
        &self.0
    }
}
impl fmt::Debug for TotpSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TotpSecret([REDACTED])")
    }
}
pub struct TotpCode(Zeroizing<[u8; 6]>);
impl TotpCode {
    pub fn new(value: &str) -> Result<Self, MfaError> {
        if value.len() != 6 || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(MfaError);
        }
        let mut bytes = Zeroizing::new([0; 6]);
        bytes.copy_from_slice(value.as_bytes());
        Ok(Self(bytes))
    }
    fn text(&self) -> Result<&str, MfaError> {
        std::str::from_utf8(&self.0[..]).map_err(|_| MfaError)
    }
}
impl fmt::Debug for TotpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TotpCode([REDACTED])")
    }
}
pub struct Enrollment {
    pub factor: FactorId,
    pub secret: TotpSecret,
}
impl fmt::Debug for Enrollment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Enrollment([REDACTED])")
    }
}
pub struct RecoveryCode(SessionToken);
impl RecoveryCode {
    pub fn from_bytes(value: &[u8]) -> Option<Self> {
        SessionToken::from_bytes(value).map(Self)
    }
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose()
    }
}
impl fmt::Debug for RecoveryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryCode([REDACTED])")
    }
}
pub struct RecoveryPermit(SessionToken);
impl RecoveryPermit {
    pub fn from_bytes(value: &[u8]) -> Option<Self> {
        SessionToken::from_bytes(value).map(Self)
    }
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose()
    }
}
impl fmt::Debug for RecoveryPermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryPermit([REDACTED])")
    }
}
pub struct RecoveryCodes(Vec<RecoveryCode>);
impl RecoveryCodes {
    pub fn codes(&self) -> &[RecoveryCode] {
        &self.0
    }
}
impl fmt::Debug for RecoveryCodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryCodes([REDACTED])")
    }
}
pub struct MfaEvidence(IdentityId);
impl MfaEvidence {
    pub fn identity(&self) -> IdentityId {
        self.0
    }
}
impl fmt::Debug for MfaEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MfaEvidence([REDACTED])")
    }
}
/// Current local state only. KC-006F must recheck it for every protected authentication purpose.
#[derive(Debug, PartialEq, Eq)]
pub struct MfaStatus {
    pub recovery_required: bool,
    pub totp_active: bool,
}
#[derive(Clone, Copy)]
pub struct MfaLimits {
    enrollment: u32,
    recovery: u32,
    codes: u8,
    skew: u8,
    attempts: AttemptLimits,
}
impl MfaLimits {
    pub fn new(
        enrollment_seconds: u32,
        recovery_seconds: u32,
        recovery_codes: u8,
        skew_steps: u8,
        attempts: AttemptLimits,
    ) -> Result<Self, MfaError> {
        if enrollment_seconds == 0
            || recovery_seconds == 0
            || !(1..=16).contains(&recovery_codes)
            || skew_steps > 1
        {
            return Err(MfaError);
        }
        Ok(Self {
            enrollment: enrollment_seconds,
            recovery: recovery_seconds,
            codes: recovery_codes,
            skew: skew_steps,
            attempts,
        })
    }
}
pub struct MfaService {
    key: Arc<EphemeralMfaKey>,
    limits: MfaLimits,
}
struct Account {
    recovery: bool,
    generation: Option<Vec<u8>>,
    permit: Option<Vec<u8>>,
    deadline: Option<u64>,
    observed: u64,
}
fn autocommit(connection: &PgConnection) -> Result<(), MfaError> {
    if connection.is_in_transaction() {
        Err(MfaError)
    } else {
        Ok(())
    }
}
async fn now(connection: &mut PgConnection) -> Result<u64, MfaError> {
    let value: String = sqlx::query_scalar(
        "SELECT pg_catalog.floor(EXTRACT(epoch FROM pg_catalog.clock_timestamp()))::text",
    )
    .fetch_one(connection)
    .await?;
    let time: u64 = value.parse().map_err(|_| MfaError)?;
    if time > u64::MAX - 60 {
        return Err(MfaError);
    }
    Ok(time)
}
async fn account(connection: &mut PgConnection, identity: IdentityId) -> Result<Account, MfaError> {
    let row = sqlx::query("SELECT recovery_required, generation, permit_hash, permit_expires::text, last_observed::text FROM identity_access.mfa_accounts WHERE identity_id = $1::text::numeric FOR UPDATE")
        .bind(identity.value().to_string()).fetch_one(&mut *connection).await?;
    let observed = row
        .try_get::<String, _>("last_observed")?
        .parse()
        .map_err(|_| MfaError)?;
    let deadline = row
        .try_get::<Option<String>, _>("permit_expires")?
        .map(|v| v.parse().map_err(|_| MfaError))
        .transpose()?;
    Ok(Account {
        recovery: row.try_get("recovery_required")?,
        generation: row.try_get("generation")?,
        permit: row.try_get("permit_hash")?,
        deadline,
        observed,
    })
}
async fn time(connection: &mut PgConnection, account: &Account) -> Result<u64, MfaError> {
    let time = now(connection).await?;
    if time < account.observed {
        return Err(MfaError);
    }
    Ok(time)
}
async fn observe(
    connection: &mut PgConnection,
    identity: IdentityId,
    time: u64,
) -> Result<(), MfaError> {
    let result = sqlx::query("UPDATE identity_access.mfa_accounts SET last_observed = $2::text::numeric WHERE identity_id = $1::text::numeric")
        .bind(identity.value().to_string()).bind(time.to_string()).execute(connection).await?;
    if result.rows_affected() != 1 {
        return Err(MfaError);
    }
    Ok(())
}
fn permit(account: &Account, time: u64, proof: Option<&RecoveryPermit>) -> Result<(), MfaError> {
    if account.recovery {
        let proof = proof.ok_or(MfaError)?;
        if account.deadline.is_none_or(|deadline| time >= deadline)
            || account.permit.as_deref() != Some(proof.0.fingerprint().as_slice())
        {
            return Err(MfaError);
        }
    } else if proof.is_some() {
        return Err(MfaError);
    }
    Ok(())
}
fn aad(identity: IdentityId, factor: FactorId, key: &[u8; 16]) -> Vec<u8> {
    [
        b"KitaConnect/totp/v1".as_slice(),
        &identity.value().to_be_bytes(),
        &factor.0,
        key,
    ]
    .concat()
}
fn matched(secret: &[u8; 20], code: &TotpCode, time: u64, skew: u8) -> Result<u64, MfaError> {
    let totp = Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_step_duration(30)
        .with_skew(u16::from(skew))
        .with_secret(Secret::new_stack(*secret))
        .build()
        .map_err(|_| MfaError)?;
    totp.check(code.text()?, time).ok_or(MfaError)
}
fn recovery_hash(identity: IdentityId, generation: &[u8], code: &RecoveryCode) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KitaConnect/recovery/v1");
    hash.update(identity.value().to_be_bytes());
    hash.update(generation);
    hash.update(code.expose());
    hash.finalize().into()
}
async fn event(
    connection: &mut PgConnection,
    identity: IdentityId,
    kind: &str,
    time: u64,
) -> Result<(), MfaError> {
    let contact: Option<String> = sqlx::query_scalar("SELECT contact_ref::text FROM identity_access.mfa_notification_contacts WHERE identity_id = $1::text::numeric AND confirmed_at <= $2::text::numeric FOR SHARE")
        .bind(identity.value().to_string()).bind(time.to_string()).fetch_optional(&mut *connection).await?;
    let contact = contact.ok_or(MfaError)?;
    let id = random::<16>()?;
    let written = sqlx::query("INSERT INTO identity_access.mfa_security_events (id, identity_id, kind, created_at) VALUES ($1, $2::text::numeric, $3, $4::text::numeric)")
        .bind(id.as_slice()).bind(identity.value().to_string()).bind(kind).bind(time.to_string()).execute(&mut *connection).await?;
    if written.rows_affected() != 1 {
        return Err(MfaError);
    }
    let written = sqlx::query("INSERT INTO identity_access.mfa_notification_outbox (event_id, contact_ref) VALUES ($1, $2::text::uuid)").bind(id.as_slice()).bind(contact).execute(connection).await?;
    if written.rows_affected() != 1 {
        return Err(MfaError);
    }
    Ok(())
}
async fn revoke_sessions(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<(), MfaError> {
    // Shared lock order: MFA account -> WebAuthn account -> session rows -> events.
    let passkey: Option<String> = sqlx::query_scalar("SELECT identity_id::text FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric FOR UPDATE")
        .bind(identity.value().to_string()).fetch_optional(&mut *connection).await?;
    if passkey.is_some() {
        let updated = sqlx::query("UPDATE identity_access.webauthn_accounts SET revision = $2 WHERE identity_id = $1::text::numeric")
            .bind(identity.value().to_string()).bind(random::<32>()?.as_slice()).execute(&mut *connection).await?;
        if updated.rows_affected() != 1 {
            return Err(MfaError);
        }
    }
    let sessions = sqlx::query("SELECT id FROM identity_access.sessions WHERE identity_id = $1::text::numeric AND state <> 'revoked' FOR UPDATE")
        .bind(identity.value().to_string()).fetch_all(&mut *connection).await?;
    let updated = sqlx::query("UPDATE identity_access.sessions SET state = 'revoked' WHERE identity_id = $1::text::numeric AND state <> 'revoked'")
        .bind(identity.value().to_string()).execute(connection).await?;
    if updated.rows_affected() != sessions.len() as u64 {
        return Err(MfaError);
    }
    Ok(())
}

impl MfaService {
    pub fn new(key: Arc<EphemeralMfaKey>, limits: MfaLimits) -> Self {
        Self { key, limits }
    }
    async fn reserve(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
    ) -> Result<(), MfaError> {
        let mut hash = Sha256::new();
        hash.update(b"KitaConnect/mfa-attempt/v1");
        hash.update(identity.value().to_be_bytes());
        password::reserve(
            connection,
            AttemptKey::from_server_key(hash.finalize().into()),
            self.limits.attempts,
        )
        .await
        .map_err(|_| MfaError)
    }
    async fn nonce(&self, connection: &mut PgConnection) -> Result<[u8; 12], MfaError> {
        autocommit(connection)?;
        let value: i64 = sqlx::query_scalar("INSERT INTO identity_access.mfa_nonce_counters (key_tag, used) VALUES ($1, 1) ON CONFLICT (key_tag) DO UPDATE SET used = identity_access.mfa_nonce_counters.used + 1 WHERE identity_access.mfa_nonce_counters.used < 4294967295 RETURNING used")
            .bind(self.key.tag.as_slice()).fetch_one(connection).await?;
        let mut nonce = [0; 12];
        nonce[4..].copy_from_slice(&value.to_be_bytes());
        Ok(nonce)
    }
    fn open(
        &self,
        identity: IdentityId,
        row: &sqlx::postgres::PgRow,
    ) -> Result<TotpSecret, MfaError> {
        let tag: Vec<u8> = row.try_get("key_tag")?;
        let id: Vec<u8> = row.try_get("factor_id")?;
        let factor = FactorId(id.try_into().map_err(|_| MfaError)?);
        if tag != self.key.tag {
            return Err(MfaError);
        }
        let nonce: Vec<u8> = row.try_get("nonce")?;
        let nonce: [u8; 12] = nonce.try_into().map_err(|_| MfaError)?;
        let ciphertext: Vec<u8> = row.try_get("ciphertext")?;
        let cipher = Aes256Gcm::new_from_slice(&self.key.bytes[..]).map_err(|_| MfaError)?;
        let plain = Zeroizing::new(
            cipher
                .decrypt(
                    &Nonce::from(nonce),
                    Payload {
                        msg: &ciphertext,
                        aad: &aad(identity, factor, &self.key.tag),
                    },
                )
                .map_err(|_| MfaError)?,
        );
        let bytes: [u8; 20] = plain.as_slice().try_into().map_err(|_| MfaError)?;
        Ok(TotpSecret(Zeroizing::new(bytes)))
    }
    fn check(
        &self,
        identity: IdentityId,
        row: &sqlx::postgres::PgRow,
        code: &TotpCode,
        time: u64,
    ) -> Result<u64, MfaError> {
        let secret = self.open(identity, row)?;
        let step = matched(secret.expose(), code, time, self.limits.skew)?;
        let used = row
            .try_get::<Option<String>, _>("last_step")?
            .map(|v| v.parse::<u64>().map_err(|_| MfaError))
            .transpose()?;
        if used.is_some_and(|used| step <= used) {
            return Err(MfaError);
        }
        Ok(step)
    }
    /// Already verified enrollment purpose required. Recovery requires its current limited permit.
    pub async fn begin_enrollment(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        proof: Option<&RecoveryPermit>,
    ) -> Result<Enrollment, MfaError> {
        let nonce = self.nonce(connection).await?; // Durable even if enrollment later fails.
        let factor = FactorId(random()?);
        let secret = TotpSecret(Zeroizing::new(random()?));
        let cipher = Aes256Gcm::new_from_slice(&self.key.bytes[..]).map_err(|_| MfaError)?;
        let ciphertext = cipher
            .encrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: secret.expose(),
                    aad: &aad(identity, factor, &self.key.tag),
                },
            )
            .map_err(|_| MfaError)?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        crate::login::lock_identity(&mut tx, identity)
            .await
            .map_err(|_| MfaError)?;
        sqlx::query("INSERT INTO identity_access.mfa_accounts (identity_id, last_observed, recovery_required) VALUES ($1::text::numeric, 0, false) ON CONFLICT DO NOTHING")
            .bind(identity.value().to_string()).execute(&mut *tx).await?;
        let account = account(&mut tx, identity).await?;
        let time = time(&mut tx, &account).await?;
        permit(&account, time, proof)?;
        sqlx::query("DELETE FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND state = 'pending'").bind(identity.value().to_string()).execute(&mut *tx).await?;
        let written = sqlx::query("INSERT INTO identity_access.totp_factors (factor_id, identity_id, state, key_tag, nonce, ciphertext, created_at, expires_at) VALUES ($1, $2::text::numeric, 'pending', $3, $4, $5, $6::text::numeric, $7::text::numeric)")
            .bind(factor.0.as_slice()).bind(identity.value().to_string()).bind(self.key.tag.as_slice()).bind(nonce.as_slice()).bind(ciphertext)
            .bind(time.to_string()).bind(time.checked_add(u64::from(self.limits.enrollment)).ok_or(MfaError)?.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        observe(&mut tx, identity, time).await?;
        event(&mut tx, identity, "enrollment", time).await?;
        tx.commit().await?;
        Ok(Enrollment { factor, secret })
    }
    async fn issue_codes(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
    ) -> Result<RecoveryCodes, MfaError> {
        let generation = random::<16>()?;
        sqlx::query(
            "DELETE FROM identity_access.mfa_recovery_codes WHERE identity_id = $1::text::numeric",
        )
        .bind(identity.value().to_string())
        .execute(&mut *connection)
        .await?;
        let written = sqlx::query("UPDATE identity_access.mfa_accounts SET generation = $2, recovery_required = false, permit_hash = NULL, permit_expires = NULL WHERE identity_id = $1::text::numeric")
            .bind(identity.value().to_string()).bind(generation.as_slice()).execute(&mut *connection).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        let mut codes = Vec::new();
        for _ in 0..self.limits.codes {
            let code = RecoveryCode(SessionToken::generate().map_err(|_| MfaError)?);
            let written = sqlx::query("INSERT INTO identity_access.mfa_recovery_codes (fingerprint, identity_id, generation, consumed) VALUES ($1, $2::text::numeric, $3, false)")
                .bind(recovery_hash(identity, &generation, &code).as_slice()).bind(identity.value().to_string()).bind(generation.as_slice()).execute(&mut *connection).await?;
            if written.rows_affected() != 1 {
                return Err(MfaError);
            }
            codes.push(code);
        }
        Ok(RecoveryCodes(codes))
    }
    /// Replacing an active TOTP also requires a fresh code of the previous factor.
    pub async fn activate(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        enrollment: &Enrollment,
        code: &TotpCode,
        previous: Option<&TotpCode>,
        proof: Option<&RecoveryPermit>,
    ) -> Result<RecoveryCodes, MfaError> {
        self.reserve(connection, identity).await?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        crate::login::lock_identity(&mut tx, identity)
            .await
            .map_err(|_| MfaError)?;
        let account = account(&mut tx, identity).await?;
        let time = time(&mut tx, &account).await?;
        permit(&account, time, proof)?;
        let active = sqlx::query("SELECT factor_id, key_tag, nonce, ciphertext, last_step::text FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND state = 'active'")
            .bind(identity.value().to_string()).fetch_optional(&mut *tx).await?;
        if let Some(row) = active {
            self.check(identity, &row, previous.ok_or(MfaError)?, time)?;
        } else if previous.is_some() {
            return Err(MfaError);
        }
        let row = sqlx::query("SELECT factor_id, key_tag, nonce, ciphertext, last_step::text, created_at::text, expires_at::text FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND factor_id = $2 AND state = 'pending'")
            .bind(identity.value().to_string()).bind(enrollment.factor.0.as_slice()).fetch_one(&mut *tx).await?;
        let created: u64 = row
            .try_get::<String, _>("created_at")?
            .parse()
            .map_err(|_| MfaError)?;
        let expires: u64 = row
            .try_get::<String, _>("expires_at")?
            .parse()
            .map_err(|_| MfaError)?;
        if time < created || time >= expires {
            return Err(MfaError);
        }
        let step = self.check(identity, &row, code, time)?;
        sqlx::query("DELETE FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND state = 'active'").bind(identity.value().to_string()).execute(&mut *tx).await?;
        let written = sqlx::query("UPDATE identity_access.totp_factors SET state = 'active', last_step = $2::text::numeric WHERE factor_id = $1")
            .bind(enrollment.factor.0.as_slice()).bind(step.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        let codes = self.issue_codes(&mut tx, identity).await?;
        observe(&mut tx, identity, time).await?;
        revoke_sessions(&mut tx, identity).await?;
        event(&mut tx, identity, "activation", time).await?;
        tx.commit().await?;
        Ok(codes)
    }
    pub async fn verify(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        code: &TotpCode,
    ) -> Result<MfaEvidence, MfaError> {
        self.verify_inner(connection, identity, code, None).await
    }
    pub async fn verify_for_login(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        code: &TotpCode,
        flow: &crate::login::LoginToken,
    ) -> Result<(), MfaError> {
        self.verify_inner(connection, identity, code, Some(flow))
            .await
            .map(|_| ())
    }
    async fn verify_inner(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        code: &TotpCode,
        flow: Option<&crate::login::LoginToken>,
    ) -> Result<MfaEvidence, MfaError> {
        self.reserve(connection, identity).await?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        crate::login::lock_identity(&mut tx, identity)
            .await
            .map_err(|_| MfaError)?;
        let account = account(&mut tx, identity).await?;
        if account.recovery {
            return Err(MfaError);
        }
        let time = time(&mut tx, &account).await?;
        let row = sqlx::query("SELECT factor_id, key_tag, nonce, ciphertext, last_step::text FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND state = 'active'")
            .bind(identity.value().to_string()).fetch_one(&mut *tx).await?;
        let step = self.check(identity, &row, code, time)?;
        let written = sqlx::query("UPDATE identity_access.totp_factors SET last_step = $2::text::numeric WHERE factor_id = $1")
            .bind(row.try_get::<Vec<u8>, _>("factor_id")?).bind(step.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        observe(&mut tx, identity, time).await?;
        if let Some(flow) = flow {
            let recorded =
                crate::login::record_factor(&mut tx, identity, flow, crate::login::Factor::Totp)
                    .await
                    .map_err(|_| MfaError)?;
            match recorded {
                crate::login::FactorRecord::Recorded => {}
                crate::login::FactorRecord::Expired => {
                    tx.commit().await?;
                    return Err(MfaError);
                }
            }
        }
        tx.commit().await?;
        Ok(MfaEvidence(identity))
    }
    pub async fn recover(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        code: &RecoveryCode,
    ) -> Result<RecoveryPermit, MfaError> {
        self.reserve(connection, identity).await?;
        let proof = RecoveryPermit(SessionToken::generate().map_err(|_| MfaError)?);
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        crate::login::lock_identity(&mut tx, identity)
            .await
            .map_err(|_| MfaError)?;
        let account = account(&mut tx, identity).await?;
        let time = time(&mut tx, &account).await?;
        let generation = account.generation.ok_or(MfaError)?;
        let written = sqlx::query("UPDATE identity_access.mfa_recovery_codes SET consumed = true WHERE fingerprint = $1 AND identity_id = $2::text::numeric AND generation = $3 AND NOT consumed")
            .bind(recovery_hash(identity, &generation, code).as_slice()).bind(identity.value().to_string()).bind(&generation).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        sqlx::query(
            "DELETE FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric",
        )
        .bind(identity.value().to_string())
        .execute(&mut *tx)
        .await?;
        let written = sqlx::query("UPDATE identity_access.mfa_accounts SET recovery_required = true, permit_hash = $2, permit_expires = $3::text::numeric WHERE identity_id = $1::text::numeric")
            .bind(identity.value().to_string()).bind(proof.0.fingerprint().as_slice()).bind(time.checked_add(u64::from(self.limits.recovery)).ok_or(MfaError)?.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(MfaError);
        }
        observe(&mut tx, identity, time).await?;
        revoke_sessions(&mut tx, identity).await?;
        event(&mut tx, identity, "recovery", time).await?;
        tx.commit().await?;
        Ok(proof)
    }
    pub async fn revoke(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
        code: &TotpCode,
    ) -> Result<(), MfaError> {
        self.reserve(connection, identity).await?;
        let mut tx = connection
            .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
            .await?;
        crate::login::lock_identity(&mut tx, identity)
            .await
            .map_err(|_| MfaError)?;
        let account = account(&mut tx, identity).await?;
        if account.recovery {
            return Err(MfaError);
        }
        let time = time(&mut tx, &account).await?;
        let row = sqlx::query("SELECT factor_id, key_tag, nonce, ciphertext, last_step::text FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric AND state = 'active'")
            .bind(identity.value().to_string()).fetch_one(&mut *tx).await?;
        self.check(identity, &row, code, time)?;
        sqlx::query(
            "DELETE FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric",
        )
        .bind(identity.value().to_string())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM identity_access.mfa_recovery_codes WHERE identity_id = $1::text::numeric",
        )
        .bind(identity.value().to_string())
        .execute(&mut *tx)
        .await?;
        let updated = sqlx::query("UPDATE identity_access.mfa_accounts SET generation = NULL, permit_hash = NULL, permit_expires = NULL WHERE identity_id = $1::text::numeric")
            .bind(identity.value().to_string()).execute(&mut *tx).await?;
        if updated.rows_affected() != 1 {
            return Err(MfaError);
        }
        observe(&mut tx, identity, time).await?;
        revoke_sessions(&mut tx, identity).await?;
        event(&mut tx, identity, "revocation", time).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn status(
        &self,
        connection: &mut PgConnection,
        identity: IdentityId,
    ) -> Result<MfaStatus, MfaError> {
        autocommit(connection)?;
        let row = sqlx::query("SELECT recovery_required, EXISTS(SELECT FROM identity_access.totp_factors f WHERE f.identity_id = a.identity_id AND state = 'active') AS active FROM identity_access.mfa_accounts a WHERE identity_id = $1::text::numeric")
            .bind(identity.value().to_string()).fetch_one(connection).await?;
        Ok(MfaStatus {
            recovery_required: row.try_get("recovery_required")?,
            totp_active: row.try_get("active")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_rfc6238_vectors_and_window_boundaries() {
        // RFC 6238 Appendix B public SHA-1 vector, truncated to this profile's six digits.
        let secret = *b"12345678901234567890";
        for (time, value) in [
            (59, "287082"),
            (1_111_111_109, "081804"),
            (1_111_111_111, "050471"),
            (1_234_567_890, "005924"),
            (2_000_000_000, "279037"),
            (20_000_000_000, "353130"),
        ] {
            assert_eq!(
                matched(&secret, &TotpCode::new(value).unwrap(), time, 0).unwrap(),
                time / 30
            );
        }
        let code = TotpCode::new("287082").unwrap();
        for time in [30, 59] {
            assert_eq!(matched(&secret, &code, time, 0).unwrap(), 1);
        }
        for time in [29, 60] {
            assert!(matched(&secret, &code, time, 0).is_err());
        }
        for time in [0, 89] {
            assert_eq!(matched(&secret, &code, time, 1).unwrap(), 1);
        }
        assert!(matched(&secret, &code, 90, 1).is_err());
    }
    #[test]
    fn input_limits_and_debug_do_not_expose_secrets() {
        for value in ["", "12345", "1234567", "+12345", "１２３", "12 456"] {
            assert!(TotpCode::new(value).is_err());
        }
        assert_eq!(
            format!("{:?}", TotpCode::new("123456").unwrap()),
            "TotpCode([REDACTED])"
        );
        assert!(RecoveryCode::from_bytes(&[0; 31]).is_none());
        assert!(RecoveryPermit::from_bytes(&[0; 33]).is_none());
        let attempts = AttemptLimits::new(60, 10, 20, 4).unwrap();
        for (enrollment, recovery, count, skew) in [
            (0, 1, 1, 0),
            (1, 0, 1, 0),
            (1, 1, 0, 0),
            (1, 1, 17, 0),
            (1, 1, 1, 2),
        ] {
            assert!(MfaLimits::new(enrollment, recovery, count, skew, attempts).is_err());
        }
        assert_eq!(
            format!("{:?}", EphemeralMfaKey::generate().unwrap()),
            "EphemeralMfaKey([REDACTED])"
        );
    }
}
