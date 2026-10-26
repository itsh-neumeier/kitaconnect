//! Synthetic trusted port + actual factor library/PostgreSQL. Network proof is separate.
use super::*;
use kitaconnect_identity_access::{
    session_check::{self, Requirement},
    session_rotation,
};

#[derive(Clone)]
struct Authority {
    inner: SyntheticOrganization,
    member: u64,
    rights: u64,
    complete: bool,
}
impl OrganizationAuthority for Authority {
    async fn current(
        &mut self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<OrganizationContext, LoginError> {
        let context = self.inner.current(identity, facility).await?;
        if self.complete {
            context.with_source_versions(self.member, self.rights)
        } else {
            Ok(context)
        }
    }
}
async fn database(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .username(user)
            .database("kc_source_bindings_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn check(c: &mut PgConnection, token: &SessionToken, authority: &mut Authority) -> bool {
    session_check::check(c, token, facility(1), Requirement::Standard, authority)
        .await
        .is_ok()
}
async fn versions(c: &mut PgConnection, record: u128) -> (String, String, String, bool) {
    sqlx::query_as("SELECT membership_version::text,policy_version::text,rights_version::text,raw_mfa_required FROM identity_access.session_source_observations WHERE session_id=$1::text::numeric")
        .bind(record.to_string()).fetch_one(c).await.unwrap()
}
async fn rejected(c: &mut PgConnection, statement: &'static str) {
    let error = sqlx::raw_sql(statement).execute(c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23514")
    );
}

#[tokio::test]
#[ignore = "requires the isolated PostgreSQL fixture"]
async fn complete_sources_bind_flows_sessions_and_rotation() {
    let mut c = database("postgres").await;
    let mut witness = database("postgres").await;
    let connection_pid = pid(&mut c).await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        assert_eq!(
            versions(&mut c, 1002).await,
            ("1".into(), "2".into(), "4".into(), false)
        );
        let revoked: String =
            sqlx::query_scalar("SELECT state FROM identity_access.sessions WHERE id=1010")
                .fetch_one(&mut c)
                .await
                .unwrap();
        assert_eq!(revoked, "revoked");
    }
    let base = if after { 2000 } else { 1000 };
    let identity = id(if after { 2 } else { 1 });
    sqlx::query("INSERT INTO identity_access.identities VALUES($1::text::numeric)")
        .bind(identity.value().to_string())
        .execute(&mut c)
        .await
        .unwrap();
    let mut authority = Authority {
        inner: SyntheticOrganization::new(identity),
        member: 1,
        rights: 1,
        complete: true,
    };
    let checker = PasswordChecker::new().await.unwrap();
    let password = SessionToken::generate().unwrap();
    checker
        .store(&mut c, identity, Password::new(password.expose()).unwrap())
        .await
        .unwrap();
    // Actual passkey registration before creating sessions; no synthetic success marker.
    let engine = LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 8).unwrap()).unwrap();
    let mut client = SoftPasskey::new(true);
    let (ceremony, options) = engine.start_registration(&mut c, identity).await.unwrap();
    let response = client
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            options.public_key,
            300_000,
        )
        .unwrap();
    engine
        .finish_registration(
            &mut c,
            identity,
            &ceremony,
            &serde_json::to_vec(&response).unwrap(),
        )
        .await
        .unwrap();

    let mut legacy = authority.clone();
    legacy.complete = false;
    assert!(
        login::begin(
            &mut c,
            identity,
            facility(1),
            LoginRoute::Password,
            limits(),
            &mut legacy
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    let flow = login::begin(
        &mut c,
        identity,
        facility(1),
        LoginRoute::Password,
        limits(),
        &mut authority,
    )
    .await
    .unwrap();
    prove_password(&checker, &mut c, identity, &password, &flow).await;
    let issued = login::finish(&mut c, &flow, session(base), &mut authority)
        .await
        .unwrap();
    assert_eq!(
        versions(&mut c, base).await,
        ("1".into(), "1".into(), "1".into(), false)
    );
    assert!(!check(&mut c, &issued, &mut legacy).await);
    let rotated = session_rotation::rotate(
        &mut c,
        &issued,
        session(base + 1),
        facility(1),
        &mut authority,
    )
    .await
    .unwrap();
    assert_eq!(
        versions(&mut c, base + 1).await,
        versions(&mut c, base).await
    );
    assert!(!check(&mut c, &issued, &mut authority).await);
    assert!(check(&mut c, &rotated, &mut authority).await);

    authority.inner.version = 2;
    authority.inner.idle = 600;
    authority.inner.absolute = 14400;
    authority.inner.step_up = 180;
    authority.rights = 2;
    assert!(check(&mut c, &rotated, &mut authority).await);
    assert_eq!(
        versions(&mut c, base + 1).await,
        ("1".into(), "2".into(), "2".into(), false)
    );
    for (policy, rights, idle, mfa) in [
        (1, 2, 600, false),
        (2, 1, 600, false),
        (2, 2, 500, false),
        (2, 2, 600, true),
    ] {
        let mut bad = authority.clone();
        bad.inner.version = policy;
        bad.rights = rights;
        bad.inner.idle = idle;
        bad.inner.mfa = mfa;
        assert!(!check(&mut c, &rotated, &mut bad).await);
        support::assert_idle(&mut witness, connection_pid).await;
    }
    authority.rights = 3;
    authority.inner.mfa = true;
    assert!(!check(&mut c, &rotated, &mut authority).await);
    authority.rights = 4;
    authority.inner.mfa = false;
    assert!(!check(&mut c, &rotated, &mut authority).await); // Existing MFA cap remains.
    let proof = login::begin_step_up(
        &mut c,
        &rotated,
        facility(1),
        LoginRoute::Passkey,
        limits(),
        &mut authority,
    )
    .await
    .unwrap();
    let (challenge, response) = assertion(&engine, &mut c, identity, &mut client).await;
    engine
        .finish_authentication_for_login(&mut c, identity, &challenge, &response, &proof)
        .await
        .unwrap();
    let strong = login::finish_step_up(
        &mut c,
        &rotated,
        &proof,
        session(base + 2),
        facility(1),
        &mut authority,
    )
    .await
    .unwrap();
    assert!(check(&mut c, &strong, &mut authority).await);
    assert_eq!(
        versions(&mut c, base + 2).await,
        ("1".into(), "2".into(), "4".into(), false)
    );

    // SQL guards independently reject regression and contradictory same-version values.
    sqlx::query("UPDATE identity_access.session_source_observations SET rights_version=rights_version WHERE session_id=$1::text::numeric").bind((base+2).to_string()).execute(&mut c).await.unwrap();
    rejected(
        &mut c,
        "UPDATE identity_access.session_source_observations SET membership_version=2",
    )
    .await;
    rejected(
        &mut c,
        "UPDATE identity_access.session_source_observations SET rights_version=1",
    )
    .await;
    rejected(
        &mut c,
        "UPDATE identity_access.session_source_observations SET raw_idle=599",
    )
    .await;
    rejected(&mut c,"UPDATE identity_access.session_source_observations SET raw_mfa_required=NOT raw_mfa_required").await;
    rejected(
        &mut c,
        "UPDATE identity_access.flow_source_bindings SET rights_version=rights_version",
    )
    .await;

    let mut runtime = database("kc_source_bindings_runtime").await;
    let visible: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.session_source_observations")
            .fetch_one(&mut runtime)
            .await
            .unwrap();
    assert_eq!(visible, 0);
    assert!(!check(&mut runtime, &strong, &mut authority).await);

    // A raw insert cannot commit a new flow without its deferred source receipt.
    rejected(&mut c,"WITH t AS (SELECT floor(EXTRACT(epoch FROM clock_timestamp())) AS n) INSERT INTO identity_access.login_flows(fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,mfa_required,created_at,expires_at,last_observed) SELECT decode(repeat('ab',32),'hex'),1,1,'password',1,900,28800,300,false,n,n+60,n FROM t").await;
    support::assert_idle(&mut witness, connection_pid).await;

    // Suppressed write must roll back both the session and flow consumption.
    let fresh = login::begin(
        &mut c,
        identity,
        facility(1),
        LoginRoute::Password,
        limits(),
        &mut authority,
    )
    .await
    .unwrap();
    prove_password(&checker, &mut c, identity, &password, &fresh).await;
    sql(&mut c,"CREATE FUNCTION identity_access.synthetic_skip_sources() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER synthetic_skip_sources BEFORE INSERT ON identity_access.session_source_observations FOR EACH ROW EXECUTE FUNCTION identity_access.synthetic_skip_sources()").await;
    assert!(
        login::finish(&mut c, &fresh, session(base + 8), &mut authority)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    let absent: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS(SELECT FROM identity_access.sessions WHERE id=$1::text::numeric)",
    )
    .bind((base + 8).to_string())
    .fetch_one(&mut witness)
    .await
    .unwrap();
    assert!(absent);
    sql(&mut c,"DROP TRIGGER synthetic_skip_sources ON identity_access.session_source_observations; DROP FUNCTION identity_access.synthetic_skip_sources()").await;

    // One real finalizer waits on the other's identity lock. Only one may consume the flow.
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mut paused = authority.clone();
    paused.inner.pause = Some((entered.clone(), release.clone()));
    let token_a = LoginToken::from_bytes(fresh.expose()).unwrap();
    let token_b = LoginToken::from_bytes(fresh.expose()).unwrap();
    let first = tokio::spawn(async move {
        let mut connection = database("postgres").await;
        login::finish(&mut connection, &token_a, session(base + 10), &mut paused).await
    });
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .unwrap();
    let mut contender = database("postgres").await;
    let contender_pid = pid(&mut contender).await;
    let mut copy = authority.clone();
    let second = tokio::spawn(async move {
        login::finish(&mut contender, &token_b, session(base + 11), &mut copy).await
    });
    wait(&mut witness, contender_pid).await;
    release.notify_one();
    let winner = tokio::time::timeout(Duration::from_secs(10), first)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(10), second)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    authority.member = 2;
    assert!(!check(&mut c, &winner, &mut authority).await);
    authority.member = 1;
    assert!(!check(&mut c, &winner, &mut authority).await); // Terminal revocation survives old data.

    // Missing rows cannot trigger a legacy fallback. Restore only the synthetic fixture afterwards.
    let original = versions(&mut c, base + 2).await;
    sqlx::query("DELETE FROM identity_access.session_source_observations WHERE session_id=$1::text::numeric").bind((base+2).to_string()).execute(&mut c).await.unwrap();
    assert!(!check(&mut c, &strong, &mut authority).await);
    sqlx::query("INSERT INTO identity_access.session_source_observations VALUES($1::text::numeric,1,2,4,600,14400,180,false)").bind((base+2).to_string()).execute(&mut c).await.unwrap();
    assert_eq!(versions(&mut c, base + 2).await, original);
    assert!(check(&mut c, &strong, &mut authority).await);
}
