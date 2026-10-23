//! Service-internal local completion. OrganizationAuthority is a trusted integration
//! port, not a browser claim. Network identity, cookies and later action checks are separate.
use crate::{IdentityId, SessionLimits, SessionRecord, SessionRecordId, token::SessionToken};
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, Row};
use std::{fmt, future::Future, num::NonZeroU128};
use zeroize::Zeroizing;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LoginError;
impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("login operation unavailable or unsuccessful")
    }
}
impl fmt::Debug for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for LoginError {}
impl From<sqlx::Error> for LoginError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoginFacilityId(NonZeroU128);
impl LoginFacilityId {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }
    pub fn value(self) -> u128 {
        self.0.get()
    }
}
pub struct LoginToken(SessionToken);
impl LoginToken {
    pub fn from_bytes(value: &[u8]) -> Option<Self> {
        SessionToken::from_bytes(value).map(Self)
    }
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose()
    }
}
impl fmt::Debug for LoginToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginToken([REDACTED])")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginRoute {
    Password,
    Passkey,
}
impl LoginRoute {
    fn text(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::Passkey => "passkey",
        }
    }
}
#[derive(Clone, Copy)]
pub struct LoginLimits {
    lifetime: u32,
    pending: u32,
    per_identity: u32,
}
impl LoginLimits {
    pub fn new(lifetime_seconds: u32, pending: u32, per_identity: u32) -> Result<Self, LoginError> {
        if !(1..=300).contains(&lifetime_seconds)
            || pending == 0
            || per_identity == 0
            || per_identity > pending
        {
            return Err(LoginError);
        }
        Ok(Self {
            lifetime: lifetime_seconds,
            pending,
            per_identity,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrganizationContext {
    pub(crate) identity: IdentityId,
    pub(crate) facility: LoginFacilityId,
    pub(crate) version: u64,
    pub(crate) idle: u64,
    pub(crate) absolute: u64,
    pub(crate) step_up: u64,
    pub(crate) mfa_required: bool,
}
impl OrganizationContext {
    /// Trusted adapter must establish current membership before constructing a context.
    pub fn new(
        identity: IdentityId,
        facility: LoginFacilityId,
        version: u64,
        idle_seconds: u64,
        absolute_seconds: u64,
        step_up_seconds: u64,
        mfa_required: bool,
    ) -> Result<Self, LoginError> {
        if version == 0
            || !(1..=900).contains(&idle_seconds)
            || !(1..=28_800).contains(&absolute_seconds)
            || !(1..=300).contains(&step_up_seconds)
            || idle_seconds > absolute_seconds
            || step_up_seconds > absolute_seconds
        {
            return Err(LoginError);
        }
        Ok(Self {
            identity,
            facility,
            version,
            idle: idle_seconds,
            absolute: absolute_seconds,
            step_up: step_up_seconds,
            mfa_required,
        })
    }
    pub(crate) fn bound(
        self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<Self, LoginError> {
        if self.identity != identity || self.facility != facility {
            Err(LoginError)
        } else {
            Ok(self)
        }
    }
}
/// Production implementations must authenticate their peer and read current membership/
/// settings. No callback into identity writes while a completion holds its identity lock.
pub trait OrganizationAuthority: Send {
    fn current(
        &mut self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> impl Future<Output = Result<OrganizationContext, LoginError>> + Send;
}

fn autocommit(connection: &PgConnection) -> Result<(), LoginError> {
    if connection.is_in_transaction() {
        Err(LoginError)
    } else {
        Ok(())
    }
}
async fn now(connection: &mut PgConnection) -> Result<u64, LoginError> {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(connection)
        .await?
        .parse()
        .map_err(|_| LoginError)
}
fn number(row: &sqlx::postgres::PgRow, name: &str) -> Result<u64, LoginError> {
    row.try_get::<String, _>(name)?
        .parse()
        .map_err(|_| LoginError)
}
/// Shared across every supported factor writer, using only migration-0001 objects.
pub(crate) async fn lock_identity(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<(), LoginError> {
    if !connection.is_in_transaction() {
        return Err(LoginError);
    }
    let mut hash = Sha256::new();
    hash.update(b"KitaConnect/identity-lock/v1");
    hash.update(identity.value().to_be_bytes());
    let digest: [u8; 32] = hash.finalize().into();
    let key = i64::from_be_bytes(digest[..8].try_into().map_err(|_| LoginError)?);
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(key)
        .execute(&mut *connection)
        .await?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM identity_access.identities WHERE id=$1::text::numeric)",
    )
    .bind(identity.value().to_string())
    .fetch_one(connection)
    .await?;
    if !exists {
        return Err(LoginError);
    }
    Ok(())
}
pub(crate) async fn revoke_sessions(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<(), LoginError> {
    let rows=sqlx::query("SELECT id FROM identity_access.sessions WHERE identity_id=$1::text::numeric AND state<>'revoked' FOR UPDATE")
        .bind(identity.value().to_string()).fetch_all(&mut *connection).await?;
    let updated=sqlx::query("UPDATE identity_access.sessions SET state='revoked' WHERE identity_id=$1::text::numeric AND state<>'revoked'")
        .bind(identity.value().to_string()).execute(connection).await?;
    if updated.rows_affected() != rows.len() as u64 {
        return Err(LoginError);
    }
    Ok(())
}
fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
struct Material {
    digest: [u8; 32],
    recovery: bool,
    totp: bool,
}
async fn material(
    connection: &mut PgConnection,
    identity: IdentityId,
) -> Result<Material, LoginError> {
    let mut hash = Sha256::new();
    field(&mut hash, b"KitaConnect/login-material/v1");
    field(&mut hash, &identity.value().to_be_bytes());
    let password=sqlx::query("SELECT phc, revision FROM identity_access.password_credentials WHERE identity_id=$1::text::numeric")
        .bind(identity.value().to_string()).fetch_optional(&mut *connection).await?;
    field(&mut hash, b"password");
    field(&mut hash, &[u8::from(password.is_some())]);
    if let Some(row) = password {
        let phc = Zeroizing::new(row.try_get::<String, _>("phc")?);
        field(&mut hash, phc.as_bytes());
        field(&mut hash, &row.try_get::<Vec<u8>, _>("revision")?);
    }
    let account=sqlx::query_scalar::<_,Vec<u8>>("SELECT revision FROM identity_access.webauthn_accounts WHERE identity_id=$1::text::numeric")
        .bind(identity.value().to_string()).fetch_optional(&mut *connection).await?;
    field(&mut hash, b"passkey");
    field(&mut hash, &[u8::from(account.is_some())]);
    if let Some(revision) = &account {
        field(&mut hash, revision);
    }
    let keys=sqlx::query("SELECT credential_id,data FROM identity_access.webauthn_credentials WHERE identity_id=$1::text::numeric ORDER BY credential_id LIMIT 33")
        .bind(identity.value().to_string()).fetch_all(&mut *connection).await?;
    if keys.len() > 32 || (account.is_none() && !keys.is_empty()) {
        return Err(LoginError);
    }
    field(&mut hash, &(keys.len() as u64).to_be_bytes());
    for row in keys {
        field(&mut hash, &row.try_get::<Vec<u8>, _>("credential_id")?);
        field(&mut hash, row.try_get::<String, _>("data")?.as_bytes());
    }
    let recovery:Option<bool>=sqlx::query_scalar("SELECT recovery_required FROM identity_access.mfa_accounts WHERE identity_id=$1::text::numeric")
        .bind(identity.value().to_string()).fetch_optional(&mut *connection).await?;
    let recovery = recovery.unwrap_or(false);
    field(&mut hash, b"mfa");
    field(&mut hash, &[u8::from(recovery)]);
    let active=sqlx::query("SELECT factor_id,key_tag,nonce,ciphertext FROM identity_access.totp_factors WHERE identity_id=$1::text::numeric AND state='active'")
        .bind(identity.value().to_string()).fetch_optional(connection).await?;
    let totp = active.is_some();
    field(&mut hash, &[u8::from(totp)]);
    if let Some(row) = active {
        for name in ["factor_id", "key_tag", "nonce", "ciphertext"] {
            field(&mut hash, &row.try_get::<Vec<u8>, _>(name)?);
        }
    }
    Ok(Material {
        digest: hash.finalize().into(),
        recovery,
        totp,
    })
}
struct Flow {
    identity: IdentityId,
    facility: LoginFacilityId,
    route: LoginRoute,
    context: OrganizationContext,
    created: u64,
    expires: u64,
    observed: u64,
    consumed: bool,
    invalidated: bool,
    digest: Option<Vec<u8>>,
    password: bool,
    totp: bool,
    passkey: bool,
    proven: Option<u64>,
}
impl Flow {
    fn decode(row: &sqlx::postgres::PgRow) -> Result<Self, LoginError> {
        let identity = IdentityId::new(
            row.try_get::<String, _>("identity_id")?
                .parse()
                .map_err(|_| LoginError)?,
        )
        .ok_or(LoginError)?;
        let facility = LoginFacilityId::new(
            row.try_get::<String, _>("facility_id")?
                .parse()
                .map_err(|_| LoginError)?,
        )
        .ok_or(LoginError)?;
        let route = match row.try_get::<&str, _>("route")? {
            "password" => LoginRoute::Password,
            "passkey" => LoginRoute::Passkey,
            _ => return Err(LoginError),
        };
        Ok(Self {
            identity,
            facility,
            route,
            context: OrganizationContext::new(
                identity,
                facility,
                number(row, "policy_version")?,
                number(row, "idle_seconds")?,
                number(row, "absolute_seconds")?,
                number(row, "step_up_seconds")?,
                row.try_get("mfa_required")?,
            )?,
            created: number(row, "created_at")?,
            expires: number(row, "expires_at")?,
            observed: number(row, "last_observed")?,
            consumed: row.try_get("consumed")?,
            invalidated: row.try_get("invalidated")?,
            digest: row.try_get("proof_digest")?,
            password: row.try_get("password_ok")?,
            totp: row.try_get("totp_ok")?,
            passkey: row.try_get("passkey_ok")?,
            proven: row
                .try_get::<Option<String>, _>("proven_at")?
                .map(|s| s.parse().map_err(|_| LoginError))
                .transpose()?,
        })
    }
    fn live(&self, time: u64) -> Result<(), LoginError> {
        if self.consumed
            || self.invalidated
            || time < self.created
            || time < self.observed
            || time >= self.expires
        {
            Err(LoginError)
        } else {
            Ok(())
        }
    }
}
async fn flow(connection: &mut PgConnection, token: &LoginToken) -> Result<Flow, LoginError> {
    let row=sqlx::query("SELECT identity_id::text, facility_id::text, route, policy_version::text, idle_seconds::text, absolute_seconds::text, step_up_seconds::text, mfa_required, created_at::text, expires_at::text, last_observed::text, consumed, invalidated, proof_digest, password_ok, totp_ok, passkey_ok, proven_at::text FROM identity_access.login_flows WHERE fingerprint=$1 FOR UPDATE")
        .bind(token.0.fingerprint().as_slice()).fetch_one(connection).await?;
    Flow::decode(&row)
}
#[derive(Clone, Copy)]
pub(crate) enum Factor {
    Password,
    Totp,
    Passkey,
}
#[must_use]
pub(crate) enum FactorRecord {
    Recorded,
    Expired,
}
async fn retire_expired(
    connection: &mut PgConnection,
    token: &LoginToken,
    flow: &Flow,
    time: u64,
) -> Result<bool, LoginError> {
    if flow.consumed
        || flow.invalidated
        || time < flow.observed
        || time < flow.created
        || time < flow.expires
    {
        return Ok(false);
    }
    let retired:bool=sqlx::query_scalar("UPDATE identity_access.login_flows SET invalidated=true,last_observed=$2::text::numeric WHERE fingerprint=$1 RETURNING invalidated")
        .bind(token.0.fingerprint().as_slice()).bind(time.to_string()).fetch_one(connection).await?;
    if !retired {
        return Err(LoginError);
    }
    Ok(true)
}
/// Called only from actual verifier success paths, before their transaction commits.
pub(crate) async fn record_factor(
    connection: &mut PgConnection,
    identity: IdentityId,
    token: &LoginToken,
    factor: Factor,
) -> Result<FactorRecord, LoginError> {
    let flow = flow(connection, token).await?;
    let time = now(connection).await?;
    if flow.identity != identity {
        return Err(LoginError);
    }
    if retire_expired(connection, token, &flow, time).await? {
        return Ok(FactorRecord::Expired);
    }
    flow.live(time)?;
    let material = material(connection, identity).await?;
    if material.recovery
        || flow
            .digest
            .as_deref()
            .is_some_and(|digest| digest != material.digest)
    {
        return Err(LoginError);
    }
    let (password, totp, passkey) = match factor {
        Factor::Password if flow.route == LoginRoute::Password && !flow.password => {
            (true, false, false)
        }
        Factor::Totp if flow.route == LoginRoute::Password && flow.password && !flow.totp => {
            (true, true, false)
        }
        Factor::Passkey if flow.route == LoginRoute::Passkey && !flow.passkey => {
            (false, false, true)
        }
        _ => return Err(LoginError),
    };
    let proven = flow.proven.unwrap_or(time);
    let written=sqlx::query("UPDATE identity_access.login_flows SET proof_digest=$2,password_ok=$3,totp_ok=$4,passkey_ok=$5,proven_at=$6::text::numeric,last_observed=$7::text::numeric WHERE fingerprint=$1 AND NOT consumed RETURNING proof_digest,password_ok,totp_ok,passkey_ok,proven_at::text")
        .bind(token.0.fingerprint().as_slice()).bind(material.digest.as_slice()).bind(password).bind(totp).bind(passkey).bind(proven.to_string()).bind(time.to_string()).fetch_one(connection).await?;
    if written.try_get::<Vec<u8>, _>("proof_digest")? != material.digest
        || written.try_get::<bool, _>("password_ok")? != password
        || written.try_get::<bool, _>("totp_ok")? != totp
        || written.try_get::<bool, _>("passkey_ok")? != passkey
        || number(&written, "proven_at")? != proven
    {
        return Err(LoginError);
    }
    Ok(FactorRecord::Recorded)
}

pub async fn begin(
    connection: &mut PgConnection,
    identity: IdentityId,
    facility: LoginFacilityId,
    route: LoginRoute,
    limits: LoginLimits,
    authority: &mut impl OrganizationAuthority,
) -> Result<LoginToken, LoginError> {
    autocommit(connection)?;
    let context = authority
        .current(identity, facility)
        .await?
        .bound(identity, facility)?;
    let token = LoginToken(SessionToken::generate().map_err(|_| LoginError)?);
    let mut tx = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    let result: Result<LoginToken, LoginError> = async {
        sqlx::query("SELECT pg_advisory_xact_lock(184006,5)")
            .execute(&mut *tx)
            .await?;
        lock_identity(&mut tx, identity).await?;
        let time = now(&mut tx).await?;
        sqlx::query("INSERT INTO identity_access.login_limits (id,lifetime,pending,per_identity,last_observed) VALUES (1,$1,$2,$3,0) ON CONFLICT DO NOTHING")
            .bind(i32::try_from(limits.lifetime).map_err(|_|LoginError)?).bind(i64::from(limits.pending)).bind(i64::from(limits.per_identity)).execute(&mut *tx).await?;
        let quota=sqlx::query("SELECT lifetime,pending,per_identity,last_observed::text FROM identity_access.login_limits WHERE id=1 FOR UPDATE").fetch_one(&mut *tx).await?;
        if quota.try_get::<i32, _>("lifetime")?
            != i32::try_from(limits.lifetime).map_err(|_| LoginError)?
            || quota.try_get::<i64, _>("pending")? != i64::from(limits.pending)
            || quota.try_get::<i64, _>("per_identity")? != i64::from(limits.per_identity)
            || time < number(&quota, "last_observed")?
        {
            return Err(LoginError);
        }
        let updated = sqlx::query(
            "UPDATE identity_access.login_limits SET last_observed=$1::text::numeric WHERE id=1",
        )
        .bind(time.to_string())
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(LoginError);
        }
        let material = material(&mut tx, identity).await?;
        if material.recovery {
            return Err(LoginError);
        }
        sqlx::query("DELETE FROM identity_access.login_flows WHERE NOT consumed AND expires_at<=$1::text::numeric").bind(time.to_string()).execute(&mut *tx).await?;
        let (total,own):(i64,i64)=sqlx::query_as("SELECT count(*),count(*) FILTER(WHERE identity_id=$1::text::numeric) FROM identity_access.login_flows WHERE NOT consumed")
            .bind(identity.value().to_string()).fetch_one(&mut *tx).await?;
        if total >= i64::from(limits.pending) || own >= i64::from(limits.per_identity) {
            return Err(LoginError);
        }
        let expires = time
            .checked_add(u64::from(limits.lifetime))
            .ok_or(LoginError)?;
        let written=sqlx::query("INSERT INTO identity_access.login_flows (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,mfa_required,created_at,expires_at,last_observed) VALUES ($1,$2::text::numeric,$3::text::numeric,$4,$5::text::numeric,$6::text::numeric,$7::text::numeric,$8::text::numeric,$9,$10::text::numeric,$11::text::numeric,$10::text::numeric)")
            .bind(token.0.fingerprint().as_slice()).bind(identity.value().to_string()).bind(facility.value().to_string()).bind(route.text()).bind(context.version.to_string())
            .bind(context.idle.to_string()).bind(context.absolute.to_string()).bind(context.step_up.to_string()).bind(context.mfa_required).bind(time.to_string()).bind(expires.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(LoginError);
        }

        Ok(token)
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

/// Local completion only. Every later protected action still needs current membership,
/// recovery, facility policy and domain rights checks through authenticated service calls.
pub async fn finish(
    connection: &mut PgConnection,
    token: &LoginToken,
    session: SessionRecordId,
    authority: &mut impl OrganizationAuthority,
) -> Result<SessionToken, LoginError> {
    autocommit(connection)?;
    let mut tx = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    let result: Result<Option<SessionToken>, LoginError> = async {
        let identity: String = sqlx::query_scalar(
            "SELECT identity_id::text FROM identity_access.login_flows WHERE fingerprint=$1",
        )
        .bind(token.0.fingerprint().as_slice())
        .fetch_one(&mut *tx)
        .await?;
        let identity = IdentityId::new(identity.parse().map_err(|_| LoginError)?).ok_or(LoginError)?;
        lock_identity(&mut tx, identity).await?;
        let flow = flow(&mut tx, token).await?;
        let time = now(&mut tx).await?;
        if retire_expired(&mut tx, token, &flow, time).await? {
            return Ok(None);
        }
        flow.live(time)?;
        if flow.identity != identity {
            return Err(LoginError);
        }
        let material = material(&mut tx, identity).await?;
        if material.recovery || flow.digest.as_deref() != Some(material.digest.as_slice()) {
            return Err(LoginError);
        }
        let current = authority
            .current(identity, flow.facility)
            .await?
            .bound(identity, flow.facility)?;
        if current.version < flow.context.version
            || (current.version == flow.context.version
                && (current.idle, current.absolute, current.step_up)
                    != (
                        flow.context.idle,
                        flow.context.absolute,
                        flow.context.step_up,
                    ))
        {
            return Err(LoginError);
        }
        let strong = flow.passkey || (flow.password && flow.totp);
        if !(flow.passkey || flow.password)
            || ((flow.context.mfa_required || current.mfa_required || material.totp) && !strong)
        {
            return Err(LoginError);
        }
        let time = now(&mut tx).await?;
        if retire_expired(&mut tx, token, &flow, time).await? {
            return Ok(None);
        }
        flow.live(time)?;
        let idle = flow.context.idle.min(current.idle);
        let absolute = flow.context.absolute.min(current.absolute);
        let step_up = flow.context.step_up.min(current.step_up);
        let limits = SessionLimits::new(idle, absolute).map_err(|_| LoginError)?;
        let record = SessionRecord::new(session, identity, time, limits).map_err(|_| LoginError)?;
        let session_token = SessionToken::generate().map_err(|_| LoginError)?;
        let consumed:bool=sqlx::query_scalar("UPDATE identity_access.login_flows SET consumed=true,last_observed=$2::text::numeric WHERE fingerprint=$1 AND NOT consumed RETURNING consumed")
            .bind(token.0.fingerprint().as_slice()).bind(time.to_string()).fetch_one(&mut *tx).await?;
        if !consumed {
            return Err(LoginError);
        }
        let written=sqlx::query("INSERT INTO identity_access.sessions (id,identity_id,fingerprint,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,state) VALUES ($1::text::numeric,$2::text::numeric,$3,$4::text::numeric,$5::text::numeric,$6::text::numeric,$4::text::numeric,$4::text::numeric,'active')")
            .bind(session.value().to_string()).bind(identity.value().to_string()).bind(session_token.fingerprint().as_slice()).bind(time.to_string()).bind(record.absolute_expires_at().to_string()).bind(idle.to_string()).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(LoginError);
        }
        let written=sqlx::query("INSERT INTO identity_access.session_login_bindings (session_id,flow_fingerprint,facility_id,policy_version,step_up_seconds,authenticated_at,mfa) VALUES ($1::text::numeric,$2,$3::text::numeric,$4::text::numeric,$5::text::numeric,$6::text::numeric,$7)")
            .bind(session.value().to_string()).bind(token.0.fingerprint().as_slice()).bind(flow.facility.value().to_string()).bind(current.version.to_string()).bind(step_up.to_string()).bind(flow.proven.ok_or(LoginError)?.to_string()).bind(strong).execute(&mut *tx).await?;
        if written.rows_affected() != 1 {
            return Err(LoginError);
        }

        Ok(Some(session_token))
    }.await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            value.ok_or(LoginError)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_bounds_and_diagnostics_are_limited() {
        assert!(LoginToken::from_bytes(&[0; 31]).is_none());
        assert!(LoginToken::from_bytes(&[0; 33]).is_none());
        let token = LoginToken::from_bytes(&[9; 32]).unwrap();
        assert_eq!(format!("{token:?}"), "LoginToken([REDACTED])");
        assert_eq!(
            format!("{LoginError:?}"),
            "login operation unavailable or unsuccessful"
        );
        for (lifetime, pending, own) in [(0, 1, 1), (301, 1, 1), (1, 0, 1), (1, 1, 0), (1, 1, 2)] {
            assert!(LoginLimits::new(lifetime, pending, own).is_err());
        }
    }
    #[test]
    fn organization_context_is_bound_and_bounded() {
        let id = IdentityId::new(1).unwrap();
        let facility = LoginFacilityId::new(2).unwrap();
        let context = OrganizationContext::new(id, facility, 1, 900, 28_800, 300, true).unwrap();
        assert!(
            context
                .bound(IdentityId::new(3).unwrap(), facility)
                .is_err()
        );
        assert!(context.bound(id, LoginFacilityId::new(3).unwrap()).is_err());
        for (version, idle, absolute, step_up) in [
            (0, 1, 1, 1),
            (1, 0, 1, 1),
            (1, 1, 0, 1),
            (1, 1, 1, 0),
            (1, 901, 28_800, 1),
            (1, 1, 28_801, 1),
            (1, 1, 28_800, 301),
            (1, 2, 1, 1),
        ] {
            assert!(
                OrganizationContext::new(id, facility, version, idle, absolute, step_up, true)
                    .is_err()
            );
        }
    }
}
