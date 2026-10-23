use kitaconnect_identity_access::{
    IdentityId, SessionError, SessionLimits, SessionRecord, SessionRecordId, SessionState,
};

fn session(created: u64, idle: u64, absolute: u64) -> SessionRecord {
    SessionRecord::new(
        SessionRecordId::new(1).unwrap(),
        IdentityId::new(2).unwrap(),
        created,
        SessionLimits::new(idle, absolute).unwrap(),
    )
    .unwrap()
}

#[test]
fn record_keys_reject_zero_and_preserve_the_full_identifier() {
    assert!(IdentityId::new(0).is_none());
    assert!(SessionRecordId::new(0).is_none());
    assert_eq!(IdentityId::new(u128::MAX).unwrap().value(), u128::MAX);
    assert_eq!(SessionRecordId::new(u128::MAX).unwrap().value(), u128::MAX);
    let record = session(100, 4, 20);
    assert_eq!(record.id().value(), 1);
    assert_eq!(record.identity().value(), 2);
}

#[test]
fn zero_durations_do_not_create_implicitly_unbounded_sessions() {
    for (idle, absolute) in [(0, 0), (0, 20), (4, 0)] {
        assert_eq!(
            SessionLimits::new(idle, absolute),
            Err(SessionError::InvalidLimits)
        );
    }
}

#[test]
fn idle_expiry_is_inclusive_and_observation_does_not_extend_it() {
    let mut record = session(100, 4, 20);
    assert_eq!(record.expires_at(), 104);
    for time in 100..104 {
        assert_eq!(record.observe(time), Ok(SessionState::Active));
    }
    assert_eq!(record.last_activity_at(), 100);
    assert_eq!(record.expires_at(), 104);
    assert_eq!(record.observe(104), Ok(SessionState::Expired));
    assert_eq!(
        record.record_activity(104),
        Err(SessionError::InactiveSession)
    );
    assert_eq!(record.last_activity_at(), 100);
}

#[test]
fn activity_can_extend_idle_time_but_never_the_absolute_deadline() {
    let mut record = session(100, 4, 20);
    for time in [103, 106, 109, 112, 115, 118, 119] {
        assert_eq!(record.record_activity(time), Ok(()));
        assert_eq!(record.expires_at(), (time + 4).min(120));
        assert_eq!(record.created_at(), 100);
        assert_eq!(record.absolute_expires_at(), 120);
    }
    assert_eq!(record.observe(120), Ok(SessionState::Expired));
    assert_eq!(
        record.record_activity(120),
        Err(SessionError::InactiveSession)
    );
    assert_eq!(record.last_activity_at(), 119);
}

#[test]
fn absolute_expiry_applies_even_when_the_idle_duration_is_larger() {
    let mut record = session(100, 40, 7);
    assert_eq!(record.expires_at(), 107);
    assert_eq!(record.record_activity(106), Ok(()));
    assert_eq!(record.expires_at(), 107);
    assert_eq!(record.observe(107), Ok(SessionState::Expired));
}

#[test]
fn observed_expiry_cannot_be_reversed_by_activity_or_an_earlier_clock() {
    let mut record = session(100, 4, 20);
    assert_eq!(record.observe(104), Ok(SessionState::Expired));
    assert_eq!(
        record.record_activity(105),
        Err(SessionError::InactiveSession)
    );
    assert_eq!(record.observe(103), Err(SessionError::ClockRegression));
    assert_eq!(
        record.record_activity(102),
        Err(SessionError::ClockRegression)
    );
    assert_eq!(record.state(), SessionState::Expired);
    assert_eq!(record.last_activity_at(), 100);
    assert_eq!(record.observe(200), Ok(SessionState::Expired));
}

#[test]
fn revocation_is_permanent_and_idempotent_even_before_time_expiry() {
    let mut record = session(100, 4, 20);
    record.revoke();
    record.revoke();
    assert_eq!(record.observe(100), Ok(SessionState::Revoked));
    assert_eq!(
        record.record_activity(101),
        Err(SessionError::InactiveSession)
    );
    assert_eq!(record.last_activity_at(), 100);
    assert_eq!(record.observe(999), Ok(SessionState::Revoked));
    assert_eq!(record.state(), SessionState::Revoked);
}

#[test]
fn a_backward_clock_cannot_reset_the_last_observation_or_extend_idle_time() {
    let mut record = session(100, 4, 20);
    assert_eq!(record.record_activity(103), Ok(()));
    for time in [0, 99, 100, 102] {
        assert_eq!(record.observe(time), Err(SessionError::ClockRegression));
        assert_eq!(
            record.record_activity(time),
            Err(SessionError::ClockRegression)
        );
        assert_eq!(record.last_observed_at(), 103);
        assert_eq!(record.last_activity_at(), 103);
        assert_eq!(record.expires_at(), 107);
    }
    assert_eq!(record.record_activity(106), Ok(()));
    assert_eq!(record.observe(110), Ok(SessionState::Expired));
}

#[test]
fn creation_overflow_is_rejected_instead_of_wrapping_the_deadline() {
    let result = SessionRecord::new(
        SessionRecordId::new(1).unwrap(),
        IdentityId::new(2).unwrap(),
        u64::MAX - 1,
        SessionLimits::new(1, 2).unwrap(),
    );
    assert_eq!(result, Err(SessionError::TimeOverflow));
}

#[test]
fn idle_extension_near_the_time_limit_remains_bounded_without_overflow() {
    let mut record = session(u64::MAX - 10, 9, 10);
    assert_eq!(record.record_activity(u64::MAX - 2), Ok(()));
    assert_eq!(record.expires_at(), u64::MAX);
    assert_eq!(record.observe(u64::MAX - 1), Ok(SessionState::Active));
    assert_eq!(record.observe(u64::MAX), Ok(SessionState::Expired));
}

#[test]
fn maximum_explicit_durations_do_not_create_an_unbounded_record() {
    let mut record = session(0, u64::MAX, u64::MAX);
    assert_eq!(record.record_activity(1), Ok(()));
    assert_eq!(record.expires_at(), u64::MAX);
    assert_eq!(record.observe(u64::MAX), Ok(SessionState::Expired));
}
