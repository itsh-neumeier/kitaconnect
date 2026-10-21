use kitaconnect_identity_access::{
    IdentityId,
    mfa::{Enrollment, EphemeralMfaKey, MfaLimits, MfaService, RecoveryCode, TotpCode},
    password::AttemptLimits,
};
use sqlx::{
    AssertSqlSafe, Connection, PgConnection, Row,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use totp_rs::{Builder, Secret};
use zeroize::Zeroizing;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
            .username(user)
            .database("kc_mfa_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
fn id(value: u128) -> IdentityId {
    IdentityId::new(value).unwrap()
}
fn engine() -> MfaService {
    MfaService::new(
        Arc::new(EphemeralMfaKey::generate().unwrap()),
        MfaLimits::new(
            300,
            300,
            4,
            1,
            AttemptLimits::new(3600, 200, 2000, 100).unwrap(),
        )
        .unwrap(),
    )
}
async fn sql(c: &mut PgConnection, value: &'static str) {
    sqlx::raw_sql(value).execute(c).await.unwrap();
}
async fn count(c: &mut PgConnection, value: &str) -> i64 {
    // Only fixed fixture identifiers and locally generated numeric IDs enter this helper.
    sqlx::query_scalar(AssertSqlSafe(value))
        .fetch_one(c)
        .await
        .unwrap()
}
async fn time(c: &mut PgConnection) -> u64 {
    let value: String =
        sqlx::query_scalar("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
            .fetch_one(c)
            .await
            .unwrap();
    value.parse().unwrap()
}
fn code(enrollment: &Enrollment, time: u64) -> TotpCode {
    let totp = Builder::new()
        .with_secret(Secret::new_stack(*enrollment.secret.expose()))
        .build()
        .unwrap();
    let value = Zeroizing::new(totp.generate(time).to_string());
    TotpCode::new(&value).unwrap()
}
async fn reset_step(c: &mut PgConnection, identity: IdentityId) {
    // Isolated administrator fixture only; the adapter never clears consumed steps.
    sqlx::query("UPDATE identity_access.totp_factors SET last_step = NULL WHERE identity_id = $1::text::numeric")
        .bind(identity.value().to_string()).execute(c).await.unwrap();
}
async fn seed_session(c: &mut PgConnection, identity: IdentityId) {
    let token = kitaconnect_identity_access::token::SessionToken::generate().unwrap();
    sqlx::query("INSERT INTO identity_access.sessions VALUES ($1::text::numeric, $1::text::numeric, $2, 1, 18446744073709551615, 100, 1, 1, 'active')")
        .bind(identity.value().to_string()).bind(token.expose().as_slice()).execute(c).await.unwrap();
}
async fn wait_for_lock(c: &mut PgConnection, pid: i32) {
    for _ in 0..200 {
        let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid = $1 AND wait_event_type = 'Lock')")
            .bind(pid).fetch_one(&mut *c).await.unwrap();
        if waiting {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("MFA contender did not reach its database lock");
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires isolated PostgreSQL fixture"]
async fn totp_and_recovery_are_atomic_and_protected() {
    let mut c = connect("postgres").await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        assert_eq!(count(&mut c, "SELECT count(*) FROM identity_access.mfa_accounts WHERE identity_id = 1 AND recovery_required").await, 1);
        assert_eq!(count(&mut c, "SELECT count(*) FROM identity_access.mfa_recovery_codes WHERE identity_id = 1 AND consumed").await, 1);
        assert!(
            count(
                &mut c,
                "SELECT count(*) FROM identity_access.mfa_nonce_counters WHERE used > 0"
            )
            .await
                > 0
        );
        assert_eq!(count(&mut c, "SELECT count(*) FROM identity_access.mfa_security_events e LEFT JOIN identity_access.mfa_notification_outbox o ON o.event_id = e.id WHERE o.event_id IS NULL").await, 0);
        // Prior process keys were never persisted or imported. A new key cannot open old factors.
        let old = engine();
        assert!(
            old.verify(&mut c, id(2), &TotpCode::new("000000").unwrap())
                .await
                .is_err()
        );
    }
    let base = if after { 100 } else { 0 };
    let a = id(base + 1);
    let b = id(base + 2);
    let absent = id(base + 3);
    for identity in [a, b, absent] {
        sqlx::query("INSERT INTO identity_access.identities VALUES ($1::text::numeric)")
            .bind(identity.value().to_string())
            .execute(&mut c)
            .await
            .unwrap();
    }
    for identity in [a, b] {
        sqlx::query("INSERT INTO identity_access.mfa_notification_contacts VALUES ($1::text::numeric, '00000000-0000-4000-8000-000000000001', 0)")
            .bind(identity.value().to_string()).execute(&mut c).await.unwrap();
    }
    let service = Arc::new(engine());
    assert!(
        service
            .begin_enrollment(&mut c, absent, None)
            .await
            .is_err()
    );
    assert_eq!(
        count(
            &mut c,
            &format!(
                "SELECT count(*) FROM identity_access.mfa_accounts WHERE identity_id = {}",
                absent.value()
            )
        )
        .await,
        0
    );
    let first = service.begin_enrollment(&mut c, a, None).await.unwrap();
    assert!(!service.status(&mut c, a).await.unwrap().totp_active);
    let initial_time = time(&mut c).await;
    assert!(
        service
            .verify(&mut c, a, &code(&first, initial_time))
            .await
            .is_err()
    );
    let snapshot = sqlx::query("SELECT factor_id, ciphertext, nonce, key_tag FROM identity_access.totp_factors WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).fetch_one(&mut c).await.unwrap();
    let ciphertext: Vec<u8> = snapshot.get("ciphertext");
    assert_eq!(ciphertext.len(), 36);
    assert!(!ciphertext.windows(20).any(|v| v == first.secret.expose()));
    let future = time(&mut c).await + 30;
    let initial_codes = service
        .activate(&mut c, a, &first, &code(&first, future), None, None)
        .await
        .unwrap();
    assert_eq!(initial_codes.codes().len(), 4);
    assert!(
        service
            .verify(&mut c, a, &code(&first, future))
            .await
            .is_err()
    );
    // Encryption integrity, key availability and AAD identity/factor binding.
    reset_step(&mut c, a).await;
    assert!(
        engine()
            .verify(&mut c, a, &code(&first, future))
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.totp_factors SET ciphertext = set_byte(ciphertext, 0, get_byte(ciphertext, 0) # 1) WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        service
            .verify(&mut c, a, &code(&first, future))
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.totp_factors SET ciphertext = $2 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).bind(&ciphertext).execute(&mut c).await.unwrap();
    sqlx::query("UPDATE identity_access.totp_factors SET factor_id = $2 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).bind([7u8;16].as_slice()).execute(&mut c).await.unwrap();
    assert!(
        service
            .verify(&mut c, a, &code(&first, future))
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.totp_factors SET factor_id = $2 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).bind(snapshot.get::<Vec<u8>,_>("factor_id")).execute(&mut c).await.unwrap();
    sqlx::query("INSERT INTO identity_access.mfa_accounts (identity_id,last_observed,recovery_required) VALUES ($1::text::numeric,0,false)")
        .bind(absent.value().to_string()).execute(&mut c).await.unwrap();
    sqlx::query("UPDATE identity_access.totp_factors SET identity_id = $2::text::numeric WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).bind(absent.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        service
            .verify(&mut c, absent, &code(&first, future))
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.totp_factors SET identity_id = $2::text::numeric WHERE identity_id = $1::text::numeric")
        .bind(absent.value().to_string()).bind(a.value().to_string()).execute(&mut c).await.unwrap();
    // Actual contending row locks: exactly one verification consumes the step.
    reset_step(&mut c, a).await;
    let mut guard = connect("postgres").await;
    let mut tx = guard.begin().await.unwrap();
    sqlx::query(
        "SELECT FROM identity_access.mfa_accounts WHERE identity_id = $1::text::numeric FOR UPDATE",
    )
    .bind(a.value().to_string())
    .execute(&mut *tx)
    .await
    .unwrap();
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let left_pid = pid(&mut left).await;
    let right_pid = pid(&mut right).await;
    let s = service.clone();
    let left_code = code(&first, future);
    let l = tokio::spawn(async move { s.verify(&mut left, a, &left_code).await });
    let s = service.clone();
    let right_code = code(&first, future);
    let r = tokio::spawn(async move { s.verify(&mut right, a, &right_code).await });
    wait_for_lock(&mut c, left_pid).await;
    wait_for_lock(&mut c, right_pid).await;
    tx.commit().await.unwrap();
    assert_eq!(
        usize::from(l.await.unwrap().is_ok()) + usize::from(r.await.unwrap().is_ok()),
        1
    );
    // Time regression and pending expiration fail closed.
    sqlx::query("UPDATE identity_access.mfa_accounts SET last_observed = 18446744073709551615 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        service
            .verify(&mut c, a, &code(&first, future))
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.mfa_accounts SET last_observed = 0 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap();
    let replacement = service.begin_enrollment(&mut c, a, None).await.unwrap();
    reset_step(&mut c, a).await;
    assert!(
        service
            .activate(
                &mut c,
                a,
                &replacement,
                &code(&replacement, future),
                None,
                None
            )
            .await
            .is_err()
    );
    sqlx::query("UPDATE identity_access.totp_factors SET created_at = 0, expires_at = 1 WHERE identity_id = $1::text::numeric AND state = 'pending'")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        service
            .activate(
                &mut c,
                a,
                &replacement,
                &code(&replacement, future),
                Some(&code(&first, future)),
                None
            )
            .await
            .is_err()
    );
    let replacement = service.begin_enrollment(&mut c, a, None).await.unwrap();
    let issued_new = service
        .activate(
            &mut c,
            a,
            &replacement,
            &code(&replacement, future),
            Some(&code(&first, future)),
            None,
        )
        .await
        .unwrap();
    assert!(
        service
            .recover(&mut c, a, &initial_codes.codes()[0])
            .await
            .is_err()
    );
    // Foreign identities cannot consume the same raw recovery code.
    let second = service.begin_enrollment(&mut c, b, None).await.unwrap();
    let second_codes = service
        .activate(&mut c, b, &second, &code(&second, future), None, None)
        .await
        .unwrap();
    assert!(
        service
            .recover(&mut c, b, &issued_new.codes()[0])
            .await
            .is_err()
    );
    seed_session(&mut c, a).await;
    sqlx::query("INSERT INTO identity_access.webauthn_accounts VALUES ($1::text::numeric, '00000000-0000-4000-8000-000000000001', $2, 0)")
        .bind(a.value().to_string()).bind([1u8;32].as_slice()).execute(&mut c).await.unwrap();
    // A suppressed outbox insert must roll back consumption, factor deletion and session revocation.
    sql(&mut c,"CREATE FUNCTION public.kc_mfa_suppress() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER kc_mfa_suppress BEFORE INSERT ON identity_access.mfa_notification_outbox FOR EACH ROW EXECUTE FUNCTION public.kc_mfa_suppress();").await;
    let before_events = count(
        &mut c,
        "SELECT count(*) FROM identity_access.mfa_security_events",
    )
    .await;
    assert!(
        service
            .recover(&mut c, a, &issued_new.codes()[0])
            .await
            .is_err()
    );
    assert!(!service.status(&mut c, a).await.unwrap().recovery_required);
    assert!(service.status(&mut c, a).await.unwrap().totp_active);
    assert_eq!(
        count(
            &mut c,
            "SELECT count(*) FROM identity_access.mfa_security_events"
        )
        .await,
        before_events
    );
    assert_eq!(count(&mut c,&format!("SELECT count(*) FROM identity_access.sessions WHERE identity_id = {} AND state = 'active'",a.value())).await,1);
    sql(&mut c,"DROP TRIGGER kc_mfa_suppress ON identity_access.mfa_notification_outbox; DROP FUNCTION public.kc_mfa_suppress();").await;
    // Recovery contenders wait on the same account; one raw code succeeds at most once.
    let mut tx = guard.begin().await.unwrap();
    sqlx::query(
        "SELECT FROM identity_access.mfa_accounts WHERE identity_id = $1::text::numeric FOR UPDATE",
    )
    .bind(a.value().to_string())
    .execute(&mut *tx)
    .await
    .unwrap();
    let mut left = connect("postgres").await;
    let mut right = connect("postgres").await;
    let left_pid = pid(&mut left).await;
    let right_pid = pid(&mut right).await;
    let s = service.clone();
    let raw = RecoveryCode::from_bytes(issued_new.codes()[0].expose()).unwrap();
    let l = tokio::spawn(async move { s.recover(&mut left, a, &raw).await });
    let s = service.clone();
    let raw = RecoveryCode::from_bytes(issued_new.codes()[0].expose()).unwrap();
    let r = tokio::spawn(async move { s.recover(&mut right, a, &raw).await });
    wait_for_lock(&mut c, left_pid).await;
    wait_for_lock(&mut c, right_pid).await;
    tx.commit().await.unwrap();
    let permit = match (l.await.unwrap(), r.await.unwrap()) {
        (Ok(p), Err(_)) | (Err(_), Ok(p)) => p,
        _ => panic!("recovery was not single-use"),
    };
    assert!(
        service
            .recover(&mut c, a, &issued_new.codes()[0])
            .await
            .is_err()
    );
    assert!(service.status(&mut c, a).await.unwrap().recovery_required);
    assert!(!service.status(&mut c, a).await.unwrap().totp_active);
    assert_eq!(count(&mut c,&format!("SELECT count(*) FROM identity_access.sessions WHERE identity_id = {} AND state = 'revoked'",a.value())).await,1);
    let revision:Vec<u8>=sqlx::query_scalar("SELECT revision FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).fetch_one(&mut c).await.unwrap();
    assert_ne!(revision, [1u8; 32]);
    assert!(service.begin_enrollment(&mut c, a, None).await.is_err());
    assert!(
        service
            .begin_enrollment(&mut c, b, Some(&permit))
            .await
            .is_err()
    );
    let recovered = service
        .begin_enrollment(&mut c, a, Some(&permit))
        .await
        .unwrap();
    sqlx::query("UPDATE identity_access.mfa_accounts SET permit_expires = 0 WHERE identity_id = $1::text::numeric")
        .bind(a.value().to_string()).execute(&mut c).await.unwrap();
    assert!(
        service
            .activate(
                &mut c,
                a,
                &recovered,
                &code(&recovered, future),
                None,
                Some(&permit)
            )
            .await
            .is_err()
    );
    // Another unused offline code can recover a lost/expired limited permit.
    let permit = service
        .recover(&mut c, a, &issued_new.codes()[1])
        .await
        .unwrap();
    let recovered = service
        .begin_enrollment(&mut c, a, Some(&permit))
        .await
        .unwrap();
    let new_codes = service
        .activate(
            &mut c,
            a,
            &recovered,
            &code(&recovered, future),
            None,
            Some(&permit),
        )
        .await
        .unwrap();
    assert!(!service.status(&mut c, a).await.unwrap().recovery_required);
    assert!(
        service
            .begin_enrollment(&mut c, a, Some(&permit))
            .await
            .is_err()
    );
    assert!(
        service
            .recover(&mut c, a, &issued_new.codes()[2])
            .await
            .is_err()
    );
    // Leave one consumed code and recovery-required state as restart evidence.
    service
        .recover(&mut c, a, &new_codes.codes()[0])
        .await
        .unwrap();
    assert_eq!(count(&mut c,"SELECT count(*) FROM identity_access.mfa_security_events e LEFT JOIN identity_access.mfa_notification_outbox o ON o.event_id = e.id WHERE o.event_id IS NULL").await,0);
    assert_eq!(count(&mut c,&format!("SELECT count(*) FROM identity_access.mfa_recovery_codes WHERE identity_id = {} AND consumed",a.value())).await,1);
    // Revocation must also invalidate its offline codes and revoke sessions atomically.
    reset_step(&mut c, b).await;
    seed_session(&mut c, b).await;
    sql(&mut c,"CREATE FUNCTION public.kc_mfa_suppress() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER kc_mfa_suppress BEFORE UPDATE ON identity_access.sessions FOR EACH ROW EXECUTE FUNCTION public.kc_mfa_suppress();").await;
    assert!(
        service
            .revoke(&mut c, b, &code(&second, future))
            .await
            .is_err()
    );
    assert!(service.status(&mut c, b).await.unwrap().totp_active);
    sql(&mut c,"DROP TRIGGER kc_mfa_suppress ON identity_access.sessions; DROP FUNCTION public.kc_mfa_suppress();").await;
    service
        .revoke(&mut c, b, &code(&second, future))
        .await
        .unwrap();
    assert!(!service.status(&mut c, b).await.unwrap().totp_active);
    assert!(
        service
            .recover(&mut c, b, &second_codes.codes()[0])
            .await
            .is_err()
    );
    let second = service.begin_enrollment(&mut c, b, None).await.unwrap();
    service
        .activate(&mut c, b, &second, &code(&second, future), None, None)
        .await
        .unwrap();
    // The trigger forbids reset. Only the fixture administrator can prepare exhaustion.
    assert!(
        sqlx::query("UPDATE identity_access.mfa_nonce_counters SET used = 1 WHERE key_tag = $1")
            .bind(snapshot.get::<Vec<u8>, _>("key_tag"))
            .execute(&mut c)
            .await
            .is_err()
    );
    sql(&mut c,"ALTER TABLE identity_access.mfa_nonce_counters DISABLE TRIGGER mfa_nonce_transition_guard;").await;
    // Deliberate nonce-budget exhaustion cannot encrypt or replace a factor.
    sqlx::query(
        "UPDATE identity_access.mfa_nonce_counters SET used = 4294967295 WHERE key_tag = $1",
    )
    .bind(snapshot.get::<Vec<u8>, _>("key_tag"))
    .execute(&mut c)
    .await
    .unwrap();
    sql(
        &mut c,
        "ALTER TABLE identity_access.mfa_nonce_counters ENABLE TRIGGER mfa_nonce_transition_guard;",
    )
    .await;
    assert!(service.begin_enrollment(&mut c, b, None).await.is_err());
    // Default-deny RLS applies to runtime and even the owning role.
    let mut runtime = connect("kc_identity_test_runtime").await;
    assert!(
        service
            .begin_enrollment(&mut runtime, b, None)
            .await
            .is_err()
    );
    for table in [
        "mfa_accounts",
        "mfa_nonce_counters",
        "totp_factors",
        "mfa_recovery_codes",
        "mfa_notification_contacts",
        "mfa_security_events",
        "mfa_notification_outbox",
    ] {
        assert_eq!(
            count(
                &mut runtime,
                &format!("SELECT count(*) FROM identity_access.{table}")
            )
            .await,
            0
        );
        let mut tx = c.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE kc_identity_owner")
            .execute(&mut *tx)
            .await
            .unwrap();
        assert_eq!(
            count(
                &mut tx,
                &format!("SELECT count(*) FROM identity_access.{table}")
            )
            .await,
            0
        );
        tx.rollback().await.unwrap();
    }
    assert_eq!(
        count(
            &mut c,
            "SELECT count(*) FROM pg_policies WHERE schemaname = 'identity_access'"
        )
        .await,
        0
    );
}
