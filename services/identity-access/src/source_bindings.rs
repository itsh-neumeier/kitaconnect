//! Service-local durable source receipts. Never domain permission or a remote transaction.
use crate::{
    SessionRecordId,
    login::{LoginError, OrganizationContext},
};
use sqlx::{PgConnection, Row};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceVersions {
    pub(crate) membership: u64,
    pub(crate) policy: u64,
    pub(crate) rights: u64,
    pub(crate) raw_mfa: bool,
}
impl SourceVersions {
    pub fn membership(&self) -> u64 {
        self.membership
    }
    pub fn policy(&self) -> u64 {
        self.policy
    }
    pub fn rights(&self) -> u64 {
        self.rights
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Record {
    sources: SourceVersions,
    raw: [u64; 3],
}
impl Record {
    fn context(context: &OrganizationContext) -> Result<Self, LoginError> {
        let sources = context.sources.ok_or(LoginError)?;
        if sources.policy != context.version
            || sources.membership == 0
            || sources.rights == 0
            || (sources.raw_mfa && !context.mfa_required)
        {
            return Err(LoginError);
        }
        Ok(Self {
            sources,
            raw: [context.idle, context.absolute, context.step_up],
        })
    }
    fn decode(row: &sqlx::postgres::PgRow) -> Result<Self, LoginError> {
        fn number(row: &sqlx::postgres::PgRow, name: &str) -> Result<u64, LoginError> {
            row.try_get::<String, _>(name)?
                .parse()
                .map_err(|_| LoginError)
        }
        let value = Self {
            sources: SourceVersions {
                membership: number(row, "membership_version")?,
                policy: number(row, "policy_version")?,
                rights: number(row, "rights_version")?,
                raw_mfa: row.try_get("raw_mfa_required")?,
            },
            raw: [
                number(row, "raw_idle")?,
                number(row, "raw_absolute")?,
                number(row, "raw_step_up")?,
            ],
        };
        if value.sources.membership == 0
            || value.sources.policy == 0
            || value.sources.rights == 0
            || !(1..=900).contains(&value.raw[0])
            || !(1..=28800).contains(&value.raw[1])
            || !(1..=300).contains(&value.raw[2])
            || value.raw[0] > value.raw[1]
            || value.raw[2] > value.raw[1]
        {
            return Err(LoginError);
        }
        Ok(value)
    }
}
pub(crate) async fn installed(c: &mut PgConnection) -> Result<bool, LoginError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM identity_access.schema_versions WHERE version>=11)",
    )
    .fetch_one(c)
    .await?)
}
async fn mode(c: &mut PgConnection, context: &OrganizationContext) -> Result<bool, LoginError> {
    let present = installed(c).await?;
    if present != context.sources.is_some() {
        return Err(LoginError);
    }
    Ok(present)
}
fn advance(old: Record, next: Record) -> Result<(), LoginError> {
    if next.sources.membership != old.sources.membership
        || next.sources.policy < old.sources.policy
        || next.sources.rights < old.sources.rights
        || (next.sources.policy == old.sources.policy && next.raw != old.raw)
        || (next.sources.rights == old.sources.rights
            && next.sources.raw_mfa != old.sources.raw_mfa)
    {
        return Err(LoginError);
    }
    Ok(())
}
pub(crate) fn successor(
    old: &OrganizationContext,
    next: &OrganizationContext,
) -> Result<(), LoginError> {
    match (old.sources, next.sources) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) => advance(Record::context(old)?, Record::context(next)?),
        _ => Err(LoginError),
    }
}
pub(crate) fn membership_changed(old: &OrganizationContext, next: &OrganizationContext) -> bool {
    matches!((old.sources,next.sources), (Some(a),Some(b)) if a.membership != b.membership)
}
pub(crate) async fn load_flow(
    c: &mut PgConnection,
    fingerprint: &[u8],
    context: &mut OrganizationContext,
) -> Result<(), LoginError> {
    if !installed(c).await? {
        return Ok(());
    }
    let row = sqlx::query("SELECT membership_version::text,policy_version::text,rights_version::text,raw_idle::text,raw_absolute::text,raw_step_up::text,raw_mfa_required FROM identity_access.flow_source_bindings WHERE flow_fingerprint=$1")
        .bind(fingerprint).fetch_one(c).await?;
    let stored = Record::decode(&row)?;
    context.sources = Some(stored.sources);
    if Record::context(context)? != stored {
        return Err(LoginError);
    }
    Ok(())
}
pub(crate) async fn require_proof(
    c: &mut PgConnection,
    fingerprint: &[u8],
) -> Result<(), LoginError> {
    if !installed(c).await? {
        return Ok(());
    }
    let row=sqlx::query("SELECT membership_version::text,policy_version::text,rights_version::text,raw_idle::text,raw_absolute::text,raw_step_up::text,raw_mfa_required FROM identity_access.flow_source_bindings WHERE flow_fingerprint=$1")
        .bind(fingerprint).fetch_one(c).await?;
    Record::decode(&row)?;
    Ok(())
}
pub(crate) async fn insert_flow(
    c: &mut PgConnection,
    fingerprint: &[u8],
    context: &OrganizationContext,
) -> Result<(), LoginError> {
    if !mode(c, context).await? {
        return Ok(());
    }
    let value = Record::context(context)?;
    let row=sqlx::query("INSERT INTO identity_access.flow_source_bindings VALUES($1,$2::text::numeric,$3::text::numeric,$4::text::numeric,$5::text::numeric,$6::text::numeric,$7::text::numeric,$8) RETURNING membership_version::text,policy_version::text,rights_version::text,raw_idle::text,raw_absolute::text,raw_step_up::text,raw_mfa_required")
        .bind(fingerprint).bind(value.sources.membership.to_string()).bind(value.sources.policy.to_string()).bind(value.sources.rights.to_string())
        .bind(value.raw[0].to_string()).bind(value.raw[1].to_string()).bind(value.raw[2].to_string()).bind(value.sources.raw_mfa).fetch_one(c).await?;
    if Record::decode(&row)? != value {
        return Err(LoginError);
    }
    Ok(())
}
pub(crate) async fn insert_session(
    c: &mut PgConnection,
    session: SessionRecordId,
    context: &OrganizationContext,
) -> Result<(), LoginError> {
    if !mode(c, context).await? {
        return Ok(());
    }
    write_session(c, session, Record::context(context)?, false).await
}
async fn session(c: &mut PgConnection, id: SessionRecordId) -> Result<Record, LoginError> {
    let row=sqlx::query("SELECT s.membership_version::text,s.policy_version::text,s.rights_version::text,s.raw_idle::text,s.raw_absolute::text,s.raw_step_up::text,s.raw_mfa_required FROM identity_access.session_source_observations s JOIN identity_access.session_login_bindings b ON b.session_id=s.session_id JOIN identity_access.flow_source_bindings f ON f.flow_fingerprint=b.flow_fingerprint WHERE s.session_id=$1::text::numeric FOR UPDATE OF s")
        .bind(id.value().to_string()).fetch_one(c).await?;
    Record::decode(&row)
}
pub(crate) async fn observe(
    c: &mut PgConnection,
    id: SessionRecordId,
    context: &OrganizationContext,
) -> Result<bool, LoginError> {
    if !mode(c, context).await? {
        return Ok(true);
    }
    let old = session(c, id).await?;
    let next = Record::context(context)?;
    if old.sources.membership != next.sources.membership {
        return Ok(false);
    }
    advance(old, next)?;
    write_session(c, id, next, true).await?;
    Ok(true)
}
async fn write_session(
    c: &mut PgConnection,
    id: SessionRecordId,
    value: Record,
    exists: bool,
) -> Result<(), LoginError> {
    let statement = if exists {
        "UPDATE identity_access.session_source_observations SET membership_version=$2::text::numeric,policy_version=$3::text::numeric,rights_version=$4::text::numeric,raw_idle=$5::text::numeric,raw_absolute=$6::text::numeric,raw_step_up=$7::text::numeric,raw_mfa_required=$8 WHERE session_id=$1::text::numeric RETURNING membership_version::text,policy_version::text,rights_version::text,raw_idle::text,raw_absolute::text,raw_step_up::text,raw_mfa_required"
    } else {
        "INSERT INTO identity_access.session_source_observations VALUES($1::text::numeric,$2::text::numeric,$3::text::numeric,$4::text::numeric,$5::text::numeric,$6::text::numeric,$7::text::numeric,$8) RETURNING membership_version::text,policy_version::text,rights_version::text,raw_idle::text,raw_absolute::text,raw_step_up::text,raw_mfa_required"
    };
    let row = sqlx::query(statement)
        .bind(id.value().to_string())
        .bind(value.sources.membership.to_string())
        .bind(value.sources.policy.to_string())
        .bind(value.sources.rights.to_string())
        .bind(value.raw[0].to_string())
        .bind(value.raw[1].to_string())
        .bind(value.raw[2].to_string())
        .bind(value.sources.raw_mfa)
        .fetch_one(c)
        .await?;
    if Record::decode(&row)? != value {
        return Err(LoginError);
    }
    Ok(())
}
pub(crate) async fn copy_session(
    c: &mut PgConnection,
    old: SessionRecordId,
    next: SessionRecordId,
) -> Result<(), LoginError> {
    if !installed(c).await? {
        return Ok(());
    }
    let value = session(c, old).await?;
    write_session(c, next, value, false).await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record() -> Record {
        Record {
            sources: SourceVersions {
                membership: 1,
                policy: 2,
                rights: 3,
                raw_mfa: false,
            },
            raw: [900, 28800, 300],
        }
    }
    #[test]
    fn independent_versions_and_same_version_values_are_enforced() {
        let old = record();
        assert!(advance(old, old).is_ok());
        for member in [0, 2, 3] {
            let mut next = old;
            next.sources.membership = member;
            assert!(advance(old, next).is_err());
        }
        let mut next = old;
        next.sources.policy = 1;
        assert!(advance(old, next).is_err());
        next = old;
        next.sources.rights = 2;
        assert!(advance(old, next).is_err());
        next = old;
        next.raw[0] = 300;
        assert!(advance(old, next).is_err());
        next.sources.policy = 3;
        assert!(advance(old, next).is_ok());
        next = old;
        next.sources.raw_mfa = true;
        assert!(advance(old, next).is_err());
        next.sources.rights = 4;
        assert!(advance(old, next).is_ok());
    }
}
