//! Internal session lifecycle and storage only: no login endpoint or authorization.
//! Record identifiers and local state must never replace a current server-side identity check.

use std::fmt;
use std::num::NonZeroU128;

pub mod password;
pub mod postgres;
pub mod token;
pub mod webauthn;

/// Internal identity key. Not a credential, token, organization key or proof of authentication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityId(NonZeroU128);

impl IdentityId {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }

    pub fn value(self) -> u128 {
        self.0.get()
    }
}

/// Internal storage key, never a browser session bearer secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionRecordId(NonZeroU128);

impl SessionRecordId {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }

    pub fn value(self) -> u128 {
        self.0.get()
    }
}

/// Caller-supplied positive durations in seconds; no project policy defaults are implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionLimits {
    idle_seconds: u64,
    absolute_seconds: u64,
}

impl SessionLimits {
    pub fn new(idle_seconds: u64, absolute_seconds: u64) -> Result<Self, SessionError> {
        if idle_seconds == 0 || absolute_seconds == 0 {
            return Err(SessionError::InvalidLimits);
        }
        Ok(Self {
            idle_seconds,
            absolute_seconds,
        })
    }
}

/// Local lifecycle state, not an authentication or access decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Expired,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionError {
    InvalidLimits,
    TimeOverflow,
    ClockRegression,
    InactiveSession,
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimits => "session durations must be positive",
            Self::TimeOverflow => "session expiry exceeds the time range",
            Self::ClockRegression => "session time moved backwards",
            Self::InactiveSession => "session record is inactive",
        })
    }
}

impl std::error::Error for SessionError {}

/// In-memory record only. Persistence must provide current, atomic lifecycle updates.
#[derive(Debug, PartialEq, Eq)]
pub struct SessionRecord {
    id: SessionRecordId,
    identity: IdentityId,
    created_at: u64,
    absolute_expires_at: u64,
    last_activity_at: u64,
    last_observed_at: u64,
    limits: SessionLimits,
    state: SessionState,
}

impl SessionRecord {
    /// Times are trusted server-side whole seconds since the Unix epoch.
    /// Identity and session-record keys cannot be interchanged:
    /// ```compile_fail
    /// use kitaconnect_identity_access::{IdentityId, SessionLimits, SessionRecord};
    /// let identity = IdentityId::new(1).unwrap();
    /// let limits = SessionLimits::new(4, 20).unwrap();
    /// SessionRecord::new(identity, identity, 100, limits);
    /// ```
    pub fn new(
        id: SessionRecordId,
        identity: IdentityId,
        created_at: u64,
        limits: SessionLimits,
    ) -> Result<Self, SessionError> {
        let absolute_expires_at = created_at
            .checked_add(limits.absolute_seconds)
            .ok_or(SessionError::TimeOverflow)?;
        Ok(Self {
            id,
            identity,
            created_at,
            absolute_expires_at,
            last_activity_at: created_at,
            last_observed_at: created_at,
            limits,
            state: SessionState::Active,
        })
    }

    pub fn id(&self) -> SessionRecordId {
        self.id
    }

    pub fn identity(&self) -> IdentityId {
        self.identity
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn absolute_expires_at(&self) -> u64 {
        self.absolute_expires_at
    }

    pub fn last_activity_at(&self) -> u64 {
        self.last_activity_at
    }

    pub fn last_observed_at(&self) -> u64 {
        self.last_observed_at
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Earlier of the idle and absolute deadlines. Bounded addition cannot overflow.
    pub fn expires_at(&self) -> u64 {
        let remaining = self.absolute_expires_at - self.last_activity_at;
        self.last_activity_at + self.limits.idle_seconds.min(remaining)
    }

    /// Observe time without recording user activity. An error is a refused check.
    pub fn observe(&mut self, now: u64) -> Result<SessionState, SessionError> {
        if now < self.last_observed_at {
            return Err(SessionError::ClockRegression);
        }
        self.last_observed_at = now;
        if self.state == SessionState::Active && now >= self.expires_at() {
            self.state = SessionState::Expired;
        }
        Ok(self.state)
    }

    /// Caller must first verify current authenticated and authorized activity.
    /// This model method is not that verification and never extends the absolute deadline.
    pub fn record_activity(&mut self, now: u64) -> Result<(), SessionError> {
        if self.observe(now)? != SessionState::Active {
            return Err(SessionError::InactiveSession);
        }
        self.last_activity_at = now;
        Ok(())
    }

    /// Permanent and idempotent within this record; stale snapshots are not current state.
    pub fn revoke(&mut self) {
        self.state = SessionState::Revoked;
    }
}
