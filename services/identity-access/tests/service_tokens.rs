use kitaconnect_identity_access::service_tokens::{
    self, AuthorityConfiguration, PeerReference, ServiceName as Name, ServiceScope as Scope,
    ServiceToken,
};
use sha2::{Digest, Sha256};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;
mod support;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .database("kc_service_token_test")
            .username(user)
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn sql(c: &mut PgConnection, text: &'static str) {
    sqlx::raw_sql(text).execute(c).await.unwrap();
}
async fn clock(c: &mut PgConnection) -> u64 {
    sqlx::query_scalar::<_, String>("SELECT floor(extract(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await
        .unwrap()
        .parse()
        .unwrap()
}
fn fingerprint(token: &ServiceToken) -> Vec<u8> {
    let mut hash = Sha256::new();
    hash.update(b"KitaConnect/service-token/v1");
    hash.update(token.expose());
    hash.finalize().to_vec()
}
async fn state(c: &mut PgConnection, token: &ServiceToken) -> String {
    sqlx::query_scalar("SELECT state FROM identity_access.service_tokens WHERE fingerprint=$1")
        .bind(fingerprint(token))
        .fetch_one(c)
        .await
        .unwrap()
}
async fn count(c: &mut PgConnection) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM identity_access.service_tokens")
        .fetch_one(c)
        .await
        .unwrap()
}
async fn pid(c: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(c)
        .await
        .unwrap()
}
async fn wait_for_lock(witness: &mut PgConnection, target: i32) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let waiting: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1 AND wait_event_type='Lock')",
        )
        .bind(target)
        .fetch_one(&mut *witness)
        .await
        .unwrap();
        if waiting {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "expected a real PostgreSQL lock waiter"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[test]
fn service_token_configuration_is_explicit() {
    for issuer in [
        "http://identity.test/",
        "https://user@identity.test/",
        "https://identity.test/?q=1",
        "https://identity.test/#f",
        "invalid",
    ] {
        assert!(AuthorityConfiguration::new([1; 16], issuer).is_err());
    }
    assert!(AuthorityConfiguration::new([0; 16], "https://identity.test/").is_err());
    assert_eq!(
        AuthorityConfiguration::new([1; 16], "https://identity.test")
            .unwrap()
            .issuer(),
        "https://identity.test/"
    );
    assert!(ServiceToken::from_bytes(&[1; 31]).is_none());
    assert!(ServiceToken::from_bytes(&[1; 33]).is_none());
    assert_eq!(
        format!("{:?}", ServiceToken::from_bytes(&[1; 32]).unwrap()),
        "ServiceToken([REDACTED])"
    );
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn service_tokens_are_current_and_bound() {
    let mut c = connect("postgres").await;
    let mut witness = connect("postgres").await;
    let connection_pid = pid(&mut c).await;
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    let cfg = AuthorityConfiguration::new([1; 16], "https://identity.test/").unwrap();
    if after {
        let terminal:i64=sqlx::query_scalar("SELECT count(*) FROM identity_access.service_tokens WHERE state IN ('expired','revoked')")
            .fetch_one(&mut c).await.unwrap();
        assert!(terminal >= 4);
    } else {
        sqlx::query("INSERT INTO identity_access.service_authority VALUES(true,$1,$2)")
            .bind(cfg.installation().as_slice())
            .bind(cfg.issuer())
            .execute(&mut c)
            .await
            .unwrap();
        sql(&mut c,"INSERT INTO identity_access.service_clients(name,version,active) VALUES('identity-access',1,true),('organization-config',1,true),('gateway',1,true);
            INSERT INTO identity_access.service_grants VALUES('identity-access','organization-config','membership:read',1,true),('gateway','identity-access','session:check',1,true);").await;
    }
    // Synthetic public digest references. These are NOT DER certificates or an mTLS test.
    let offset = if after { 40 } else { 0 };
    let identity = PeerReference::new(Name::IdentityAccess, [11 + offset; 32]);
    let organization = PeerReference::new(Name::OrganizationConfig, [22 + offset; 32]);
    let gateway = PeerReference::new(Name::Gateway, [33 + offset; 32]);
    let start = clock(&mut c).await;
    for peer in [identity, organization, gateway] {
        sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,$2,$3::text::numeric,$4::text::numeric,false)")
            .bind(peer.certificate().as_slice()).bind(peer.service().text())
            .bind((start-1).to_string()).bind((start+600).to_string()).execute(&mut c).await.unwrap();
    }
    let mut issued = service_tokens::issue_with_metadata(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        300,
    )
    .await
    .unwrap();
    assert_eq!(issued.scope(), Scope::MembershipRead);
    assert_eq!(issued.expires_at() - issued.created_at(), 300);
    let before_response = clock(&mut c).await;
    let remaining = service_tokens::remaining_lifetime(&mut c, &mut issued)
        .await
        .unwrap();
    let after_response = clock(&mut c).await;
    assert!(u64::from(remaining) <= issued.expires_at() - before_response);
    assert!(u64::from(remaining) >= issued.expires_at() - after_response);
    let token = ServiceToken::from_bytes(issued.token().expose()).unwrap();
    let active = service_tokens::introspect(&mut c, &cfg, organization, &token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(active.client(), Name::IdentityAccess);
    assert_eq!(active.target(), Name::OrganizationConfig);
    assert_eq!(active.scope(), Scope::MembershipRead);
    assert_eq!(active.certificate(), identity.certificate());
    assert_eq!(active.expires_at() - active.created_at(), 300);
    assert_eq!(active.issuer(), cfg.issuer());
    let wire = kitaconnect_identity_access::service_oauth::introspection_response(Some(&active));
    assert_eq!(wire.as_object().unwrap().len(), 8);
    assert_eq!(wire["active"], true);
    assert_eq!(wire["client_id"], active.client().text());
    assert_eq!(wire["scope"], active.scope().text());
    assert_eq!(wire["iss"], active.issuer());
    assert_eq!(wire["aud"], active.target().text());
    assert_eq!(wire["iat"], active.created_at());
    assert_eq!(wire["exp"], active.expires_at());
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    assert_eq!(wire["cnf"].as_object().unwrap().len(), 1);
    assert_eq!(
        wire["cnf"]["x5t#S256"],
        URL_SAFE_NO_PAD.encode(active.certificate())
    );
    assert!(
        !wire
            .to_string()
            .contains(kitaconnect_identity_access::service_oauth::encode_token(&token).as_str())
    );
    assert_ne!(fingerprint(&token).as_slice(), token.expose());
    let stored: Vec<u8> = sqlx::query_scalar(
        "SELECT fingerprint FROM identity_access.service_tokens WHERE fingerprint=$1",
    )
    .bind(fingerprint(&token))
    .fetch_one(&mut c)
    .await
    .unwrap();
    assert_eq!(stored, fingerprint(&token));
    let unknown = ServiceToken::from_bytes(&[7; 32]).unwrap();
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &unknown)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service_tokens::introspect(&mut c, &cfg, gateway, &token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &token)
            .await
            .unwrap()
            .is_some()
    );
    for seconds in [0, 301] {
        assert!(
            service_tokens::issue(
                &mut c,
                &cfg,
                identity,
                Name::OrganizationConfig,
                Scope::MembershipRead,
                seconds
            )
            .await
            .is_err()
        );
    }
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            identity,
            Name::Gateway,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            identity,
            Name::OrganizationConfig,
            Scope::LocalLogin,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    let wrong_peer = PeerReference::new(Name::Gateway, identity.certificate());
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            wrong_peer,
            Name::IdentityAccess,
            Scope::SessionCheck,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    for wrong_cfg in [
        AuthorityConfiguration::new([2; 16], cfg.issuer()).unwrap(),
        AuthorityConfiguration::new([1; 16], "https://other.test/").unwrap(),
    ] {
        assert!(
            service_tokens::introspect(&mut c, &wrong_cfg, organization, &token)
                .await
                .is_err()
        );
        support::assert_idle(&mut witness, connection_pid).await;
    }
    assert!(
        service_tokens::introspect(
            &mut c,
            &cfg,
            PeerReference::new(Name::OrganizationConfig, [0; 32]),
            &token
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    assert!(service_tokens::revoke(&mut c, &token).await.unwrap());
    assert!(!service_tokens::revoke(&mut c, &token).await.unwrap());
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &token)
            .await
            .unwrap()
            .is_none()
    );

    // A changed grant permanently invalidates old tokens, even after reactivation.
    let old = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        300,
    )
    .await
    .unwrap();
    sql(&mut c,"UPDATE identity_access.service_grants SET active=false,version=version+1 WHERE scope='membership:read'").await;
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &old)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(state(&mut c, &old).await, "revoked");
    sql(&mut c,"UPDATE identity_access.service_grants SET active=true,version=version+1 WHERE scope='membership:read'").await;
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &old)
            .await
            .unwrap()
            .is_none()
    );
    let fresh = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        300,
    )
    .await
    .unwrap();
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &fresh)
            .await
            .unwrap()
            .is_some()
    );

    // Real row-lock wait: a committed revocation must win over a waiting check.
    let mut locker = connect("postgres").await;
    let mut tx = locker.begin().await.unwrap();
    sqlx::query("SELECT FROM identity_access.service_tokens WHERE fingerprint=$1 FOR UPDATE")
        .bind(fingerprint(&fresh))
        .execute(&mut *tx)
        .await
        .unwrap();
    let cfg_wait = cfg.clone();
    let waiter = tokio::spawn(async move {
        let result = service_tokens::introspect(&mut c, &cfg_wait, organization, &fresh).await;
        (c, result, fresh)
    });
    wait_for_lock(&mut witness, connection_pid).await;
    sql(
        &mut tx,
        "UPDATE identity_access.service_tokens SET state='revoked' WHERE state='active'",
    )
    .await;
    tx.commit().await.unwrap();
    let (returned, result, _fresh) = waiter.await.unwrap();
    c = returned;
    assert!(result.unwrap().is_none());
    support::assert_idle(&mut witness, connection_pid).await;

    // Expiry while blocked is persisted, never returned as a positive snapshot.
    let short = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        3,
    )
    .await
    .unwrap();
    let mut tx = locker.begin().await.unwrap();
    sqlx::query("SELECT FROM identity_access.service_tokens WHERE fingerprint=$1 FOR UPDATE")
        .bind(fingerprint(&short))
        .execute(&mut *tx)
        .await
        .unwrap();
    let cfg_wait = cfg.clone();
    let waiter = tokio::spawn(async move {
        let result = service_tokens::introspect(&mut c, &cfg_wait, organization, &short).await;
        (c, result, short)
    });
    wait_for_lock(&mut witness, connection_pid).await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    tx.commit().await.unwrap();
    let (returned, result, short) = waiter.await.unwrap();
    c = returned;
    assert!(result.unwrap().is_none());
    assert_eq!(state(&mut c, &short).await, "expired");
    let error=sqlx::query("UPDATE identity_access.service_tokens SET state='active',last_observed=created_at WHERE fingerprint=$1")
        .bind(fingerprint(&short)).execute(&mut c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23514")
    );

    // Missing write confirmation and deferred commit failure cannot issue a bearer.
    let before = count(&mut c).await;
    sql(&mut c,"CREATE FUNCTION public.suppress_service_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$;
        CREATE TRIGGER suppress_service_insert BEFORE INSERT ON identity_access.service_tokens FOR EACH ROW EXECUTE FUNCTION public.suppress_service_insert();").await;
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            identity,
            Name::OrganizationConfig,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    assert_eq!(count(&mut c).await, before);
    sql(&mut c,"DROP TRIGGER suppress_service_insert ON identity_access.service_tokens; DROP FUNCTION public.suppress_service_insert();
        CREATE FUNCTION public.reject_service_commit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'synthetic commit failure'; END $$;
        CREATE CONSTRAINT TRIGGER reject_service_commit AFTER INSERT ON identity_access.service_tokens DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.reject_service_commit();").await;
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            identity,
            Name::OrganizationConfig,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    assert_eq!(count(&mut c).await, before);
    sql(&mut c,"DROP TRIGGER reject_service_commit ON identity_access.service_tokens; DROP FUNCTION public.reject_service_commit();").await;

    let registry_token = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        30,
    )
    .await
    .unwrap();
    sql(&mut c,"UPDATE identity_access.service_clients SET version=version+1 WHERE name='organization-config'").await;
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &registry_token)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(state(&mut c, &registry_token).await, "revoked");
    let update_token = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        30,
    )
    .await
    .unwrap();
    sql(&mut c,"CREATE FUNCTION public.suppress_service_update() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$;
        CREATE TRIGGER suppress_service_update BEFORE UPDATE ON identity_access.service_tokens FOR EACH ROW EXECUTE FUNCTION public.suppress_service_update();").await;
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &update_token)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    sql(&mut c,"DROP TRIGGER suppress_service_update ON identity_access.service_tokens; DROP FUNCTION public.suppress_service_update();").await;
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &update_token)
            .await
            .unwrap()
            .is_some()
    );

    // A short registered certificate caps issuance; future and overlong certificates fail.
    let time = clock(&mut c).await;
    let short_peer = PeerReference::new(Name::IdentityAccess, [14 + offset; 32]);
    sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,'identity-access',$2::text::numeric,$3::text::numeric,false)")
        .bind(short_peer.certificate().as_slice()).bind((time-1).to_string()).bind((time+30).to_string()).execute(&mut c).await.unwrap();
    let mut capped = service_tokens::issue_with_metadata(
        &mut c,
        &cfg,
        short_peer,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        300,
    )
    .await
    .unwrap();
    assert_eq!(capped.expires_at(), time + 30);
    assert!(
        service_tokens::remaining_lifetime(&mut c, &mut capped)
            .await
            .unwrap()
            <= 30
    );
    assert_eq!(
        service_tokens::introspect(&mut c, &cfg, organization, capped.token())
            .await
            .unwrap()
            .unwrap()
            .expires_at(),
        time + 30
    );
    let mut late = service_tokens::issue_with_metadata(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        2,
    )
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert!(
        service_tokens::remaining_lifetime(&mut c, &mut late)
            .await
            .is_err()
    );
    assert!(
        service_tokens::remaining_lifetime(&mut c, &mut late)
            .await
            .is_err()
    );
    let mut tx = c.begin().await.unwrap();
    assert!(
        service_tokens::remaining_lifetime(&mut tx, &mut issued)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let future_peer = PeerReference::new(Name::IdentityAccess, [15 + offset; 32]);
    let mut interrupted = service_tokens::issue_with_metadata(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        30,
    )
    .await
    .unwrap();
    let mut unavailable = connect("postgres").await;
    let unavailable_pid = pid(&mut unavailable).await;
    let terminated: bool = sqlx::query_scalar("SELECT pg_terminate_backend($1)")
        .bind(unavailable_pid)
        .fetch_one(&mut witness)
        .await
        .unwrap();
    assert!(terminated);
    assert!(
        service_tokens::remaining_lifetime(&mut unavailable, &mut interrupted)
            .await
            .is_err()
    );
    assert!(
        service_tokens::remaining_lifetime(&mut c, &mut interrupted)
            .await
            .is_err()
    );
    sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,'identity-access',$2::text::numeric,$3::text::numeric,false)")
        .bind(future_peer.certificate().as_slice()).bind((time+600).to_string()).bind((time+900).to_string()).execute(&mut c).await.unwrap();
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            future_peer,
            Name::OrganizationConfig,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    let error=sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,'identity-access',1,86402,false)")
        .bind([16+offset;32].as_slice()).execute(&mut c).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23514")
    );

    let certificate_token = service_tokens::issue(
        &mut c,
        &cfg,
        identity,
        Name::OrganizationConfig,
        Scope::MembershipRead,
        30,
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE identity_access.service_certificates SET revoked=true WHERE fingerprint=$1",
    )
    .bind(identity.certificate().as_slice())
    .execute(&mut c)
    .await
    .unwrap();
    assert!(
        service_tokens::introspect(&mut c, &cfg, organization, &certificate_token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service_tokens::issue(
            &mut c,
            &cfg,
            identity,
            Name::OrganizationConfig,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, connection_pid).await;
    let error = sqlx::query(
        "UPDATE identity_access.service_certificates SET revoked=false WHERE fingerprint=$1",
    )
    .bind(identity.certificate().as_slice())
    .execute(&mut c)
    .await
    .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23514")
    );

    let mut tx = c.begin().await.unwrap();
    assert!(
        service_tokens::introspect(&mut tx, &cfg, organization, &unknown)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let mut runtime = connect("kc_service_token_runtime").await;
    assert!(
        service_tokens::issue(
            &mut runtime,
            &cfg,
            identity,
            Name::OrganizationConfig,
            Scope::MembershipRead,
            30
        )
        .await
        .is_err()
    );
    assert!(
        service_tokens::introspect(&mut runtime, &cfg, organization, &unknown)
            .await
            .is_err()
    );
    sql(&mut c, "SET ROLE kc_identity_owner;").await;
    assert_eq!(count(&mut c).await, 0);
    sql(&mut c, "RESET ROLE;").await;
    let version: i32 =
        sqlx::query_scalar("SELECT max(version) FROM identity_access.schema_versions")
            .fetch_one(&mut c)
            .await
            .unwrap();
    assert_eq!(version, 9);
}
