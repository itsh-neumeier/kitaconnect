//! Atomic bearer replacement within the same login. Not fresh authentication.
use crate::{
    SessionRecordId, SessionState,
    login::{LoginFacilityId, OrganizationAuthority},
    session_check::{self, CheckError, Requirement},
    token::SessionToken,
};
use sqlx::{Connection, PgConnection};

/// Caller must enforce browser Origin/CSRF and domain purpose before this internal call.
pub async fn rotate(
    connection: &mut PgConnection,
    token: &SessionToken,
    successor: SessionRecordId,
    facility: LoginFacilityId,
    authority: &mut impl OrganizationAuthority,
) -> Result<SessionToken, CheckError> {
    if connection.is_in_transaction() {
        return Err(CheckError);
    }
    let mut tx = connection
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED")
        .await?;
    match rotate_in_transaction(&mut tx, token, successor, facility, authority).await {
        Ok(result) => {
            tx.commit().await?;
            result.ok_or(CheckError)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}
async fn rotate_in_transaction(
    tx: &mut PgConnection,
    token: &SessionToken,
    successor: SessionRecordId,
    facility: LoginFacilityId,
    authority: &mut impl OrganizationAuthority,
) -> Result<Option<SessionToken>, CheckError> {
    let checked = session_check::checked_in_transaction(
        &mut *tx,
        token,
        facility,
        Requirement::Standard,
        authority,
    )
    .await?;
    let Some(mut checked) = checked else {
        return Ok(None);
    };
    rotate_checked(tx, &mut checked, successor).await
}
pub(crate) async fn rotate_checked(
    tx: &mut PgConnection,
    checked: &mut session_check::CheckedSession,
    successor: SessionRecordId,
) -> Result<Option<SessionToken>, CheckError> {
    if checked.record.id == successor {
        return Err(CheckError);
    }
    let next = SessionToken::generate().map_err(|_| CheckError)?;
    // Private savepoint only: never an externally supplied transaction.
    let mut candidate = tx.begin().await?;
    checked.record.state = SessionState::Revoked;
    session_check::save_record(&mut candidate, &checked.record).await?;
    let written=sqlx::query("INSERT INTO identity_access.sessions (id,identity_id,fingerprint,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,state) SELECT $2::text::numeric,identity_id,$3,created_at,absolute_expires_at,idle_seconds,last_activity_at,last_observed_at,'active' FROM identity_access.sessions WHERE id=$1::text::numeric AND state='revoked'")
        .bind(checked.record.id.value().to_string()).bind(successor.value().to_string()).bind(next.fingerprint().as_slice()).execute(&mut *candidate).await?;
    if written.rows_affected() != 1 {
        return Err(CheckError);
    }
    let written=sqlx::query("INSERT INTO identity_access.session_login_bindings (session_id,flow_fingerprint,facility_id,policy_version,step_up_seconds,authenticated_at,mfa) SELECT $2::text::numeric,flow_fingerprint,facility_id,policy_version,step_up_seconds,authenticated_at,mfa FROM identity_access.session_login_bindings WHERE session_id=$1::text::numeric")
        .bind(checked.record.id.value().to_string()).bind(successor.value().to_string()).execute(&mut *candidate).await?;
    if written.rows_affected() != 1 {
        return Err(CheckError);
    }
    session_check::save_observation(&mut candidate, successor, &checked.observed, false).await?;
    crate::source_bindings::copy_session(&mut candidate, checked.record.id, successor)
        .await
        .map_err(|_| CheckError)?;
    let written=sqlx::query("INSERT INTO identity_access.session_rotation_links (successor,predecessor) VALUES ($1::text::numeric,$2::text::numeric)")
        .bind(successor.value().to_string()).bind(checked.record.id.value().to_string()).execute(&mut *candidate).await?;
    if written.rows_affected() != 1 {
        return Err(CheckError);
    }
    // Inspect time after any candidate insert waits. Restore only the candidate on expiry.
    checked.record.state = SessionState::Active;
    let live = checked.observe_time(session_check::now(&mut candidate).await?)?;
    if !live {
        candidate.rollback().await?;
        session_check::save_record(&mut *tx, &checked.record).await?;
        return Ok(None);
    }
    checked.record.id = successor;
    session_check::save_record(&mut candidate, &checked.record).await?;
    candidate.commit().await?;
    Ok(Some(next))
}
