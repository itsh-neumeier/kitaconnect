//! Internal facility configuration only. No session creation or authorization.
use crate::FacilityId;
use sqlx::{Connection, PgConnection, Row};
use std::{fmt, num::NonZeroU64};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PolicyError;
impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("session policy operation unavailable or unsuccessful")
    }
}
impl fmt::Debug for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for PolicyError {}
impl From<sqlx::Error> for PolicyError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionPolicyLimits {
    idle: u64,
    absolute: u64,
    step_up: u64,
}
impl SessionPolicyLimits {
    pub const MAX_IDLE_SECONDS: u64 = 900;
    pub const MAX_ABSOLUTE_SECONDS: u64 = 28_800;
    pub const MAX_STEP_UP_SECONDS: u64 = 300;
    pub fn new(
        idle_seconds: u64,
        absolute_seconds: u64,
        step_up_seconds: u64,
    ) -> Result<Self, PolicyError> {
        if !(1..=Self::MAX_IDLE_SECONDS).contains(&idle_seconds)
            || !(1..=Self::MAX_ABSOLUTE_SECONDS).contains(&absolute_seconds)
            || !(1..=Self::MAX_STEP_UP_SECONDS).contains(&step_up_seconds)
            || idle_seconds > absolute_seconds
            || step_up_seconds > absolute_seconds
        {
            return Err(PolicyError);
        }
        Ok(Self {
            idle: idle_seconds,
            absolute: absolute_seconds,
            step_up: step_up_seconds,
        })
    }
    pub fn idle_seconds(self) -> u64 {
        self.idle
    }
    pub fn absolute_seconds(self) -> u64 {
        self.absolute
    }
    pub fn step_up_seconds(self) -> u64 {
        self.step_up
    }
}
impl Default for SessionPolicyLimits {
    /// Explicitly selected approved defaults, never a fallback for a missing database row.
    fn default() -> Self {
        Self {
            idle: Self::MAX_IDLE_SECONDS,
            absolute: Self::MAX_ABSOLUTE_SECONDS,
            step_up: Self::MAX_STEP_UP_SECONDS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyVersion(NonZeroU64);
impl PolicyVersion {
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
    pub fn value(self) -> u64 {
        self.0.get()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FacilitySessionPolicy {
    facility: FacilityId,
    version: PolicyVersion,
    limits: SessionPolicyLimits,
}
impl FacilitySessionPolicy {
    pub fn facility(self) -> FacilityId {
        self.facility
    }
    pub fn version(self) -> PolicyVersion {
        self.version
    }
    pub fn limits(self) -> SessionPolicyLimits {
        self.limits
    }
}
fn autocommit(connection: &PgConnection) -> Result<(), PolicyError> {
    if connection.is_in_transaction() {
        Err(PolicyError)
    } else {
        Ok(())
    }
}
pub(crate) fn decode(row: &sqlx::postgres::PgRow) -> Result<FacilitySessionPolicy, PolicyError> {
    fn value(row: &sqlx::postgres::PgRow, name: &str) -> Result<u64, PolicyError> {
        row.try_get::<String, _>(name)?
            .parse()
            .map_err(|_| PolicyError)
    }
    let facility = row
        .try_get::<String, _>("facility_id")?
        .parse::<u128>()
        .map_err(|_| PolicyError)?;
    Ok(FacilitySessionPolicy {
        facility: FacilityId::new(facility).ok_or(PolicyError)?,
        version: PolicyVersion::new(value(row, "version")?).ok_or(PolicyError)?,
        limits: SessionPolicyLimits::new(
            value(row, "idle_seconds")?,
            value(row, "absolute_seconds")?,
            value(row, "step_up_seconds")?,
        )?,
    })
}

/// Current visible configuration. Missing/hidden rows do not select default limits.
pub async fn read(
    connection: &mut PgConnection,
    facility: FacilityId,
) -> Result<Option<FacilitySessionPolicy>, PolicyError> {
    autocommit(connection)?;
    let row = sqlx::query("SELECT facility_id::text, version::text, idle_seconds::text, absolute_seconds::text, step_up_seconds::text FROM organization_config.facility_session_policies WHERE facility_id = $1::text::numeric")
        .bind(facility.value().to_string()).fetch_optional(connection).await?;
    row.as_ref().map(decode).transpose()
}

/// The caller must check current administrative authority. None creates version 1;
/// Some(version) updates exactly that version. Success is returned only after commit.
pub async fn write(
    connection: &mut PgConnection,
    facility: FacilityId,
    expected: Option<PolicyVersion>,
    limits: SessionPolicyLimits,
) -> Result<FacilitySessionPolicy, PolicyError> {
    autocommit(connection)?;
    let next = match expected {
        None => 1,
        Some(v) => v.value().checked_add(1).ok_or(PolicyError)?,
    };
    let mut tx = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    let result: Result<FacilitySessionPolicy, PolicyError> = async {
        let row = if let Some(expected) = expected {
            sqlx::query("UPDATE organization_config.facility_session_policies SET version = $2::text::numeric, idle_seconds = $3::text::numeric, absolute_seconds = $4::text::numeric, step_up_seconds = $5::text::numeric WHERE facility_id = $1::text::numeric AND version = $6::text::numeric RETURNING facility_id::text, version::text, idle_seconds::text, absolute_seconds::text, step_up_seconds::text")
                .bind(facility.value().to_string()).bind(next.to_string()).bind(limits.idle.to_string()).bind(limits.absolute.to_string()).bind(limits.step_up.to_string()).bind(expected.value().to_string()).fetch_optional(&mut *tx).await?
        } else {
            sqlx::query("INSERT INTO organization_config.facility_session_policies (facility_id, version, idle_seconds, absolute_seconds, step_up_seconds) VALUES ($1::text::numeric, 1, $2::text::numeric, $3::text::numeric, $4::text::numeric) ON CONFLICT (facility_id) DO NOTHING RETURNING facility_id::text, version::text, idle_seconds::text, absolute_seconds::text, step_up_seconds::text")
                .bind(facility.value().to_string()).bind(limits.idle.to_string()).bind(limits.absolute.to_string()).bind(limits.step_up.to_string()).fetch_optional(&mut *tx).await?
        }.ok_or(PolicyError)?;
        let stored = decode(&row)?;
        if stored.facility != facility || stored.version.value() != next || stored.limits != limits {
            return Err(PolicyError);
        }

        Ok(stored)
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
