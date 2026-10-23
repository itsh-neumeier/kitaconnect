//! Current membership data only. No authentication, MFA decision or reusable permission.
use crate::{
    FacilityId,
    session_policy::{self, FacilitySessionPolicy},
};
use sqlx::{Connection, PgConnection, Row};
use std::{fmt, num::NonZeroU128};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityReference(NonZeroU128);
impl IdentityReference {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }
    pub fn value(self) -> u128 {
        self.0.get()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentMembership {
    identity: IdentityReference,
    facility: FacilityId,
    version: u64,
    policy: FacilitySessionPolicy,
}
impl CurrentMembership {
    pub fn identity(self) -> IdentityReference {
        self.identity
    }
    pub fn facility(self) -> FacilityId {
        self.facility
    }
    pub fn version(self) -> u64 {
        self.version
    }
    pub fn policy(self) -> FacilitySessionPolicy {
        self.policy
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MembershipError;
impl fmt::Display for MembershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("membership data unavailable or invalid")
    }
}
impl fmt::Debug for MembershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for MembershipError {}
impl From<sqlx::Error> for MembershipError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}

/// A fresh data snapshot only. Caller must authenticate its peer and check action rights.
/// The single statement joins only objects owned by organization-config.
pub async fn read_current(
    connection: &mut PgConnection,
    identity: IdentityReference,
    facility: FacilityId,
) -> Result<Option<CurrentMembership>, MembershipError> {
    if connection.is_in_transaction() {
        return Err(MembershipError);
    }
    let row=sqlx::query("SELECT trim_scale(m.identity_ref)::text AS identity_ref,trim_scale(m.version)::text AS membership_version,trim_scale(p.facility_id)::text AS facility_id,trim_scale(p.version)::text AS version,trim_scale(p.idle_seconds)::text AS idle_seconds,trim_scale(p.absolute_seconds)::text AS absolute_seconds,trim_scale(p.step_up_seconds)::text AS step_up_seconds FROM organization_config.memberships m JOIN organization_config.facilities f ON f.id=m.facility_id JOIN organization_config.facility_session_policies p ON p.facility_id=m.facility_id WHERE m.identity_ref=$1::text::numeric AND m.facility_id=$2::text::numeric AND m.active")
        .bind(identity.value().to_string()).bind(facility.value().to_string()).fetch_optional(connection).await?;
    row.map(|row| {
        let stored_identity = row
            .try_get::<String, _>("identity_ref")?
            .parse::<u128>()
            .map_err(|_| MembershipError)?;
        let version = row
            .try_get::<String, _>("membership_version")?
            .parse::<u64>()
            .map_err(|_| MembershipError)?;
        let policy = session_policy::decode(&row).map_err(|_| MembershipError)?;
        if stored_identity != identity.value() || policy.facility() != facility || version == 0 {
            return Err(MembershipError);
        }
        Ok(CurrentMembership {
            identity,
            facility,
            version,
            policy,
        })
    })
    .transpose()
}
