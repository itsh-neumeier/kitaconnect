//! Current local session checks only. Domain rights, network identity and browser controls
//! remain the caller's responsibility. Returned values are snapshots, never reusable permits.
use crate::{
    IdentityId, SessionRecord, SessionRecordId, SessionState,
    login::{self, LoginFacilityId, OrganizationAuthority, OrganizationContext},
    postgres,
    token::SessionToken,
};
use sqlx::{Connection, PgConnection, Row};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CheckError;
impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("session check unavailable or unsuccessful")
    }
}
impl fmt::Debug for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for CheckError {}
impl From<sqlx::Error> for CheckError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Requirement {
    Standard,
    MultiFactor,
    RecentMultiFactor,
}

#[derive(Debug)]
pub struct CurrentSession {
    session: SessionRecordId,
    identity: IdentityId,
    facility: LoginFacilityId,
    mfa: bool,
}
impl CurrentSession {
    pub fn session(&self) -> SessionRecordId {
        self.session
    }
    pub fn identity(&self) -> IdentityId {
        self.identity
    }
    pub fn facility(&self) -> LoginFacilityId {
        self.facility
    }
    pub fn multi_factor(&self) -> bool {
        self.mfa
    }
}
fn autocommit(c: &PgConnection) -> Result<(), CheckError> {
    if c.is_in_transaction() {
        Err(CheckError)
    } else {
        Ok(())
    }
}
pub(crate) async fn now(c: &mut PgConnection) -> Result<u64, CheckError> {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await?
        .parse()
        .map_err(|_| CheckError)
}
fn number(r: &sqlx::postgres::PgRow, name: &str) -> Result<u64, CheckError> {
    r.try_get::<String, _>(name)?
        .parse()
        .map_err(|_| CheckError)
}
async fn identity(
    c: &mut PgConnection,
    token: &SessionToken,
) -> Result<Option<IdentityId>, CheckError> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT trim_scale(identity_id)::text FROM identity_access.sessions WHERE fingerprint=$1",
    )
    .bind(token.fingerprint().as_slice())
    .fetch_optional(c)
    .await?;
    raw.map(|r| IdentityId::new(r.parse().map_err(|_| CheckError)?).ok_or(CheckError))
        .transpose()
}
async fn record(c: &mut PgConnection, token: &SessionToken) -> Result<SessionRecord, CheckError> {
    let r=sqlx::query("SELECT trim_scale(id)::text AS id,trim_scale(identity_id)::text AS identity_id,trim_scale(created_at)::text AS created_at,trim_scale(absolute_expires_at)::text AS absolute_expires_at,trim_scale(idle_seconds)::text AS idle_seconds,trim_scale(last_activity_at)::text AS last_activity_at,trim_scale(last_observed_at)::text AS last_observed_at,state FROM identity_access.sessions WHERE fingerprint=$1 FOR UPDATE")
        .bind(token.fingerprint().as_slice()).fetch_one(c).await?;
    postgres::decode(&r).map_err(|_| CheckError)
}
pub(crate) async fn save_record(c: &mut PgConnection, r: &SessionRecord) -> Result<(), CheckError> {
    let state = match r.state {
        SessionState::Active => "active",
        SessionState::Expired => "expired",
        SessionState::Revoked => "revoked",
    };
    let changed=sqlx::query("UPDATE identity_access.sessions SET last_activity_at=$2::text::numeric,last_observed_at=$3::text::numeric,state=$4 WHERE id=$1::text::numeric")
        .bind(r.id.value().to_string()).bind(r.last_activity_at.to_string()).bind(r.last_observed_at.to_string()).bind(state).execute(c).await?;
    if changed.rows_affected() != 1 {
        return Err(CheckError);
    }
    Ok(())
}
struct Binding {
    facility: LoginFacilityId,
    version: u64,
    step_up: u64,
    authenticated: u64,
    mfa: bool,
    required: bool,
}
async fn binding(c: &mut PgConnection, r: &SessionRecord) -> Result<Binding, CheckError> {
    let row=sqlx::query("SELECT b.facility_id::text,b.policy_version::text,b.step_up_seconds::text,b.authenticated_at::text,b.mfa,f.identity_id::text AS flow_identity,f.facility_id::text AS flow_facility,f.policy_version::text AS initial_version,f.proven_at::text,f.created_at::text AS flow_created,f.expires_at::text AS flow_expires,f.idle_seconds::text AS initial_idle,f.absolute_seconds::text AS initial_absolute,f.step_up_seconds::text AS initial_step,f.consumed,f.invalidated,f.password_ok,f.totp_ok,f.passkey_ok,f.mfa_required FROM identity_access.session_login_bindings b JOIN identity_access.login_flows f ON f.fingerprint=b.flow_fingerprint WHERE b.session_id=$1::text::numeric")
        .bind(r.id.value().to_string()).fetch_one(c).await?;
    let facility = LoginFacilityId::new(
        row.try_get::<String, _>("facility_id")?
            .parse()
            .map_err(|_| CheckError)?,
    )
    .ok_or(CheckError)?;
    let version = number(&row, "policy_version")?;
    let authenticated = number(&row, "authenticated_at")?;
    let step_up = number(&row, "step_up_seconds")?;
    let mfa: bool = row.try_get("mfa")?;
    let strong = row.try_get::<bool, _>("passkey_ok")?
        || (row.try_get::<bool, _>("password_ok")? && row.try_get::<bool, _>("totp_ok")?);
    if row.try_get::<String, _>("flow_identity")? != r.identity.value().to_string()
        || row.try_get::<String, _>("flow_facility")? != facility.value().to_string()
        || !row.try_get::<bool, _>("consumed")?
        || row.try_get::<bool, _>("invalidated")?
        || version == 0
        || version < number(&row, "initial_version")?
        || authenticated != number(&row, "proven_at")?
        || authenticated > r.created_at
        || authenticated < number(&row, "flow_created")?
        || authenticated >= number(&row, "flow_expires")?
        || !(1..=300).contains(&step_up)
        || step_up > number(&row, "initial_step")?
        || step_up > r.limits.absolute_seconds
        || r.limits.idle_seconds > number(&row, "initial_idle")?
        || r.limits.absolute_seconds > number(&row, "initial_absolute")?
        || strong != mfa
    {
        return Err(CheckError);
    }
    Ok(Binding {
        facility,
        version,
        step_up,
        authenticated,
        mfa,
        required: row.try_get("mfa_required")?,
    })
}
#[derive(PartialEq, Eq)]
pub(crate) struct Observation {
    version: u64,
    raw: [u64; 3],
    caps: [u64; 3],
    required: bool,
}
impl Observation {
    fn decode(r: &sqlx::postgres::PgRow) -> Result<Self, CheckError> {
        let raw = [
            u64::try_from(r.try_get::<i32, _>("raw_idle")?).map_err(|_| CheckError)?,
            u64::try_from(r.try_get::<i32, _>("raw_absolute")?).map_err(|_| CheckError)?,
            u64::try_from(r.try_get::<i32, _>("raw_step_up")?).map_err(|_| CheckError)?,
        ];
        let caps = [
            u64::try_from(r.try_get::<i32, _>("idle_cap")?).map_err(|_| CheckError)?,
            u64::try_from(r.try_get::<i32, _>("absolute_cap")?).map_err(|_| CheckError)?,
            u64::try_from(r.try_get::<i32, _>("step_up_cap")?).map_err(|_| CheckError)?,
        ];
        if raw[0] == 0
            || raw[0] > 900
            || raw[1] == 0
            || raw[1] > 28800
            || raw[2] == 0
            || raw[2] > 300
            || raw[0] > raw[1]
            || raw[2] > raw[1]
            || caps.iter().zip(raw).any(|(a, b)| *a == 0 || *a > b)
            || caps[0] > caps[1]
            || caps[2] > caps[1]
        {
            return Err(CheckError);
        }
        let version = number(r, "version")?;
        if version == 0 {
            return Err(CheckError);
        }
        Ok(Self {
            version,
            raw,
            caps,
            required: r.try_get("mfa_required")?,
        })
    }
}
async fn observation(
    c: &mut PgConnection,
    id: SessionRecordId,
) -> Result<Option<Observation>, CheckError> {
    let row=sqlx::query("SELECT version::text,raw_idle,raw_absolute,raw_step_up,idle_cap,absolute_cap,step_up_cap,mfa_required FROM identity_access.session_policy_observations WHERE session_id=$1::text::numeric FOR UPDATE")
        .bind(id.value().to_string()).fetch_optional(c).await?;
    row.as_ref().map(Observation::decode).transpose()
}
fn combine(
    r: &SessionRecord,
    b: &Binding,
    old: Option<&Observation>,
    current: &OrganizationContext,
    totp: bool,
) -> Result<Observation, CheckError> {
    let raw = [current.idle, current.absolute, current.step_up];
    if current.version < b.version
        || old.is_some_and(|o| {
            current.version < o.version || (current.version == o.version && raw != o.raw)
        })
    {
        return Err(CheckError);
    }
    let initial = [r.limits.idle_seconds, r.limits.absolute_seconds, b.step_up];
    let prior = old.map_or(initial, |o| o.caps);
    if prior.iter().zip(initial).any(|(a, b)| *a > b) {
        return Err(CheckError);
    }
    Ok(Observation {
        version: current.version,
        raw,
        caps: std::array::from_fn(|i| initial[i].min(prior[i]).min(raw[i])),
        required: b.required || current.mfa_required || totp || old.is_some_and(|o| o.required),
    })
}
pub(crate) async fn save_observation(
    c: &mut PgConnection,
    id: SessionRecordId,
    o: &Observation,
    exists: bool,
) -> Result<(), CheckError> {
    let statement = if exists {
        "UPDATE identity_access.session_policy_observations SET version=$2::text::numeric,raw_idle=$3,raw_absolute=$4,raw_step_up=$5,idle_cap=$6,absolute_cap=$7,step_up_cap=$8,mfa_required=$9 WHERE session_id=$1::text::numeric RETURNING version::text,raw_idle,raw_absolute,raw_step_up,idle_cap,absolute_cap,step_up_cap,mfa_required"
    } else {
        "INSERT INTO identity_access.session_policy_observations (session_id,version,raw_idle,raw_absolute,raw_step_up,idle_cap,absolute_cap,step_up_cap,mfa_required) VALUES ($1::text::numeric,$2::text::numeric,$3,$4,$5,$6,$7,$8,$9) RETURNING version::text,raw_idle,raw_absolute,raw_step_up,idle_cap,absolute_cap,step_up_cap,mfa_required"
    };
    let row = sqlx::query(statement)
        .bind(id.value().to_string())
        .bind(o.version.to_string())
        .bind(i32::try_from(o.raw[0]).map_err(|_| CheckError)?)
        .bind(i32::try_from(o.raw[1]).map_err(|_| CheckError)?)
        .bind(i32::try_from(o.raw[2]).map_err(|_| CheckError)?)
        .bind(i32::try_from(o.caps[0]).map_err(|_| CheckError)?)
        .bind(i32::try_from(o.caps[1]).map_err(|_| CheckError)?)
        .bind(i32::try_from(o.caps[2]).map_err(|_| CheckError)?)
        .bind(o.required)
        .fetch_one(c)
        .await?;
    if Observation::decode(&row)? != *o {
        return Err(CheckError);
    }
    Ok(())
}
/// Fresh authentication context only. This does not authorize the caller's domain action.
pub async fn check(
    c: &mut PgConnection,
    token: &SessionToken,
    facility: LoginFacilityId,
    requirement: Requirement,
    authority: &mut impl OrganizationAuthority,
) -> Result<CurrentSession, CheckError> {
    current(c, token, facility, requirement, authority, false).await
}
/// Only for currently authorized activity; never allow client polling to select this operation.
pub async fn record_activity(
    c: &mut PgConnection,
    token: &SessionToken,
    facility: LoginFacilityId,
    requirement: Requirement,
    authority: &mut impl OrganizationAuthority,
) -> Result<CurrentSession, CheckError> {
    current(c, token, facility, requirement, authority, true).await
}
async fn current(
    c: &mut PgConnection,
    token: &SessionToken,
    facility: LoginFacilityId,
    requirement: Requirement,
    authority: &mut impl OrganizationAuthority,
    activity: bool,
) -> Result<CurrentSession, CheckError> {
    autocommit(c)?;
    let mut tx = c.begin_with("BEGIN ISOLATION LEVEL READ COMMITTED").await?;
    let checked = checked_in_transaction(&mut tx, token, facility, requirement, authority).await?;
    let Some(mut checked) = checked else {
        tx.commit().await?;
        return Err(CheckError);
    };
    if activity {
        checked
            .record
            .record_activity(checked.record.last_observed_at)
            .map_err(|_| CheckError)?;
        save_record(&mut tx, &checked.record).await?;
    }
    tx.commit().await?;
    Ok(checked.context)
}
pub(crate) struct CheckedSession {
    pub(crate) record: SessionRecord,
    pub(crate) observed: Observation,
    context: CurrentSession,
}
impl CheckedSession {
    pub(crate) fn observe_time(&mut self, time: u64) -> Result<bool, CheckError> {
        self.record.observe(time).map_err(|_| CheckError)?;
        let absolute = self
            .record
            .created_at
            .checked_add(self.observed.caps[1])
            .ok_or(CheckError)?;
        let idle = self
            .record
            .last_activity_at
            .checked_add(self.observed.caps[0])
            .ok_or(CheckError)?;
        if time >= absolute.min(idle) {
            self.record.state = SessionState::Expired;
        }
        Ok(self.record.state == SessionState::Active)
    }
}
pub(crate) async fn checked_in_transaction(
    c: &mut PgConnection,
    token: &SessionToken,
    facility: LoginFacilityId,
    requirement: Requirement,
    authority: &mut impl OrganizationAuthority,
) -> Result<Option<CheckedSession>, CheckError> {
    if !c.is_in_transaction() {
        return Err(CheckError);
    }
    let identity = identity(&mut *c, token).await?.ok_or(CheckError)?;
    login::lock_identity(&mut *c, identity)
        .await
        .map_err(|_| CheckError)?;
    let mut record = record(&mut *c, token).await?;
    if record.identity != identity || record.state != SessionState::Active {
        return Err(CheckError);
    }
    if record
        .observe(now(&mut *c).await?)
        .map_err(|_| CheckError)?
        != SessionState::Active
    {
        save_record(&mut *c, &record).await?;
        return Ok(None);
    }
    let b = binding(&mut *c, &record).await?;
    if b.facility != facility {
        return Err(CheckError);
    }
    let old = observation(&mut *c, record.id).await?;
    if let Some(old) = &old {
        let initial = [
            record.limits.idle_seconds,
            record.limits.absolute_seconds,
            b.step_up,
        ];
        if old.version < b.version || old.caps.iter().zip(initial).any(|(a, b)| *a > b) {
            return Err(CheckError);
        }
        let absolute = record
            .created_at
            .checked_add(old.caps[1])
            .ok_or(CheckError)?;
        let idle = record
            .last_activity_at
            .checked_add(old.caps[0])
            .ok_or(CheckError)?;
        if record.last_observed_at >= absolute.min(idle) {
            record.state = SessionState::Expired;
            save_record(&mut *c, &record).await?;
            return Ok(None);
        }
    }
    let recovery:Option<bool>=sqlx::query_scalar("SELECT recovery_required FROM identity_access.mfa_accounts WHERE identity_id=$1::text::numeric")
        .bind(identity.value().to_string()).fetch_optional(&mut *c).await?;
    if recovery == Some(true) {
        return Err(CheckError);
    }
    let totp:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM identity_access.totp_factors WHERE identity_id=$1::text::numeric AND state='active')")
        .bind(identity.value().to_string()).fetch_one(&mut *c).await?;
    let current = authority
        .current(identity, facility)
        .await
        .map_err(|_| CheckError)?
        .bound(identity, facility)
        .map_err(|_| CheckError)?;
    let observed = combine(&record, &b, old.as_ref(), &current, totp)?;
    let time = now(&mut *c).await?;
    record.observe(time).map_err(|_| CheckError)?;
    let absolute = record
        .created_at
        .checked_add(observed.caps[1])
        .ok_or(CheckError)?;
    let idle = record
        .last_activity_at
        .checked_add(observed.caps[0])
        .ok_or(CheckError)?;
    if time >= absolute.min(idle) {
        record.state = SessionState::Expired;
    }
    save_observation(&mut *c, record.id, &observed, old.is_some()).await?;
    let mfa_required = observed.required || !matches!(requirement, Requirement::Standard);
    let recent = !matches!(requirement, Requirement::RecentMultiFactor)
        || time
            < b.authenticated
                .checked_add(observed.caps[2])
                .ok_or(CheckError)?;
    let allowed = record.state == SessionState::Active && (!mfa_required || b.mfa) && recent;
    save_record(&mut *c, &record).await?;
    let result = CurrentSession {
        session: record.id,
        identity,
        facility,
        mfa: b.mfa,
    };
    Ok(if allowed {
        Some(CheckedSession {
            record,
            observed,
            context: result,
        })
    } else {
        None
    })
}
/// Local idempotent logout. Browser Origin/CSRF validation must precede this trusted call.
pub async fn logout(c: &mut PgConnection, token: &SessionToken) -> Result<(), CheckError> {
    autocommit(c)?;
    let mut tx = c.begin_with("BEGIN ISOLATION LEVEL READ COMMITTED").await?;
    let Some(identity) = identity(&mut tx, token).await? else {
        tx.commit().await?;
        return Ok(());
    };
    login::lock_identity(&mut tx, identity)
        .await
        .map_err(|_| CheckError)?;
    let record = record(&mut tx, token).await?;
    if record.identity != identity {
        return Err(CheckError);
    }
    let ids: Vec<String> = sqlx::query_scalar("SELECT s.id::text FROM identity_access.sessions s LEFT JOIN identity_access.session_login_bindings b ON b.session_id=s.id WHERE s.identity_id=$1::text::numeric AND (s.id=$2::text::numeric OR b.flow_fingerprint=(SELECT flow_fingerprint FROM identity_access.session_login_bindings WHERE session_id=$2::text::numeric)) ORDER BY s.id FOR UPDATE OF s")
        .bind(identity.value().to_string()).bind(record.id.value().to_string()).fetch_all(&mut *tx).await?;
    let written=sqlx::query("UPDATE identity_access.sessions SET state='revoked' WHERE identity_id=$1::text::numeric AND id::text=ANY($2)")
        .bind(identity.value().to_string()).bind(&ids).execute(&mut *tx).await?;
    if ids.is_empty() || written.rows_affected() != ids.len() as u64 {
        return Err(CheckError);
    }
    tx.commit().await?;
    Ok(())
}
