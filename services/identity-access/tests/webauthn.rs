use kitaconnect_identity_access::{
    IdentityId,
    webauthn::{CeremonyLimits, CeremonyToken, CredentialRef, LocalhostWebauthn, WebauthnError},
};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use webauthn_authenticator_rs::{
    AuthenticatorBackend, error::WebauthnCError, softpasskey::SoftPasskey,
};
use webauthn_rs::prelude::*;

async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").expect("isolated socket required"))
            .username(user)
            .database("kc_webauthn_test")
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
fn id(value: u128) -> IdentityId {
    IdentityId::new(value).unwrap()
}
fn engine() -> LocalhostWebauthn {
    LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 8).unwrap()).unwrap()
}
fn origin() -> Url {
    Url::parse("http://localhost:3000").unwrap()
}
fn error(error: WebauthnError) {
    assert_eq!(
        format!("{error:?}"),
        "passkey check unavailable or unsuccessful"
    );
}
// Deliberately use the library backend without its client-origin prechecks:
// the server must reject correctly signed responses from a malicious client too.
trait TestClient {
    fn do_registration(
        &mut self,
        origin: Url,
        options: CreationChallengeResponse,
    ) -> Result<RegisterPublicKeyCredential, WebauthnCError>;
    fn do_authentication(
        &mut self,
        origin: Url,
        options: RequestChallengeResponse,
    ) -> Result<PublicKeyCredential, WebauthnCError>;
}
impl TestClient for SoftPasskey {
    fn do_registration(
        &mut self,
        origin: Url,
        options: CreationChallengeResponse,
    ) -> Result<RegisterPublicKeyCredential, WebauthnCError> {
        self.perform_register(origin, options.public_key, 300_000)
    }
    fn do_authentication(
        &mut self,
        origin: Url,
        options: RequestChallengeResponse,
    ) -> Result<PublicKeyCredential, WebauthnCError> {
        self.perform_auth(origin, options.public_key, 300_000)
    }
}
fn auth() -> SoftPasskey {
    SoftPasskey::new(true)
}
fn duplicate(token: &CeremonyToken) -> CeremonyToken {
    CeremonyToken::from_bytes(token.expose()).unwrap()
}
async fn registration(
    engine: &LocalhostWebauthn,
    connection: &mut PgConnection,
    identity: IdentityId,
    auth: &mut SoftPasskey,
) -> CredentialRef {
    let (token, options) = engine
        .start_registration(connection, identity)
        .await
        .unwrap();
    let response = auth.do_registration(origin(), options).unwrap();
    let bytes = serde_json::to_vec(&response).unwrap();
    let credential = engine
        .finish_registration(connection, identity, &token, &bytes)
        .await
        .unwrap();
    error(
        engine
            .finish_registration(connection, identity, &token, &bytes)
            .await
            .unwrap_err(),
    );
    credential
}
async fn assertion(
    engine: &LocalhostWebauthn,
    connection: &mut PgConnection,
    identity: IdentityId,
    auth: &mut SoftPasskey,
) -> (CeremonyToken, Vec<u8>) {
    let (token, options) = engine
        .start_authentication(connection, identity)
        .await
        .unwrap();
    let response = auth.do_authentication(origin(), options).unwrap();
    (token, serde_json::to_vec(&response).unwrap())
}
async fn pid(connection: &mut PgConnection) -> i32 {
    sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(connection)
        .await
        .unwrap()
}
async fn waiters(connection: &mut PgConnection, pids: &[i32]) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_catalog.pg_stat_activity WHERE pid = ANY($1) AND wait_event_type = 'Lock' AND state = 'active'")
                .bind(pids).fetch_one(&mut *connection).await.unwrap();
            if count == pids.len() as i64 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("real row-lock waiters required");
}
async fn consumed(connection: &mut PgConnection, identity: IdentityId) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM identity_access.webauthn_ceremonies WHERE identity_id = $1::text::numeric AND consumed")
        .bind(identity.value().to_string()).fetch_one(connection).await.unwrap()
}

#[test]
fn input_limits_and_reference_transport_are_strict() {
    for args in [
        (0, 1, 1),
        (1, 0, 1),
        (1, 1, 0),
        (1, 1, 33),
        (u32::MAX, 1, 1),
    ] {
        assert!(CeremonyLimits::new(args.0, args.1, args.2).is_err());
    }
    assert!(CeremonyToken::from_bytes(&[0; 31]).is_none());
    assert_eq!(
        format!("{:?}", CeremonyToken::from_bytes(&[0; 32]).unwrap()),
        "CeremonyToken([REDACTED])"
    );
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the isolated PostgreSQL fixture in scripts/test-organization-db.mjs"]
async fn webauthn_ceremonies_are_current_and_single_use() {
    let mut connection = connect("postgres").await;
    let phase = std::env::var("KC_ORG_TEST_PHASE").unwrap();
    if phase == "after" {
        assert!(consumed(&mut connection, id(1)).await > 0);
        let old_key: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT FROM identity_access.webauthn_credentials WHERE identity_id = 1)",
        )
        .fetch_one(&mut connection)
        .await
        .unwrap();
        assert!(old_key);
        let versions: Vec<i32> = sqlx::query_scalar(
            "SELECT version FROM identity_access.schema_versions ORDER BY version",
        )
        .fetch_all(&mut connection)
        .await
        .unwrap();
        assert_eq!(versions, [1, 2, 3]);
    }
    let base = if phase == "before" { 0 } else { 100 };
    for identity in 1..=5 {
        sqlx::query("INSERT INTO identity_access.identities VALUES ($1::text::numeric)")
            .bind((base + identity).to_string())
            .execute(&mut connection)
            .await
            .unwrap();
    }
    let identity = id(base + 1);
    let other = id(base + 2);
    let service = Arc::new(engine());
    let mut device = auth();
    let key = registration(&service, &mut connection, identity, &mut device).await;
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    let evidence = service
        .finish_authentication(&mut connection, identity, &token, &response)
        .await
        .unwrap();
    assert_eq!(evidence.identity(), identity);
    assert_eq!(format!("{evidence:?}"), "PasskeyEvidence([REDACTED])");
    error(
        service
            .finish_authentication(&mut connection, identity, &token, &response)
            .await
            .unwrap_err(),
    );
    let passwords: i64 =
        sqlx::query_scalar("SELECT count(*) FROM identity_access.password_credentials")
            .fetch_one(&mut connection)
            .await
            .unwrap();
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.sessions")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!((passwords, sessions), (0, 0));

    // A correctly signed response from another enrolled device is still not this identity's credential.
    let mut other_device = auth();
    let other_key = registration(&service, &mut connection, other, &mut other_device).await;
    let duplicate_error = sqlx::query("INSERT INTO identity_access.webauthn_credentials (credential_id, identity_id, data) SELECT credential_id, $1::text::numeric, data FROM identity_access.webauthn_credentials WHERE identity_id = $2::text::numeric")
        .bind(other.value().to_string()).bind(identity.value().to_string()).execute(&mut connection).await.unwrap_err();
    assert_eq!(
        duplicate_error
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("23505")
    );
    let (token, options) = service
        .start_authentication(&mut connection, identity)
        .await
        .unwrap();
    let (_, other_options) = service
        .start_authentication(&mut connection, other)
        .await
        .unwrap();
    let mut foreign_options = options;
    foreign_options.public_key.allow_credentials = other_options.public_key.allow_credentials;
    let foreign = other_device
        .do_authentication(origin(), foreign_options)
        .unwrap();
    error(
        service
            .finish_authentication(
                &mut connection,
                identity,
                &token,
                &serde_json::to_vec(&foreign).unwrap(),
            )
            .await
            .unwrap_err(),
    );
    assert!(
        !service
            .revoke(&mut connection, identity, &other_key)
            .await
            .unwrap()
    );

    // Wrong origin, port, subdomain and RP-ID have authentic signatures made by the test library.
    for bad_origin in [
        "http://localhost:3001",
        "https://sub.localhost:3000",
        "https://elsewhere.invalid",
    ] {
        let (token, options) = service
            .start_authentication(&mut connection, identity)
            .await
            .unwrap();
        let answer = device
            .do_authentication(Url::parse(bad_origin).unwrap(), options)
            .unwrap();
        error(
            service
                .finish_authentication(
                    &mut connection,
                    identity,
                    &token,
                    &serde_json::to_vec(&answer).unwrap(),
                )
                .await
                .unwrap_err(),
        );
    }
    let (token, mut options) = service
        .start_authentication(&mut connection, identity)
        .await
        .unwrap();
    options.public_key.rp_id = "wrong.invalid".to_owned();
    let answer = device.do_authentication(origin(), options).unwrap();
    error(
        service
            .finish_authentication(
                &mut connection,
                identity,
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );

    // Sign a different challenge and a response without UV; neither is accepted.
    let (token, mut options) = service
        .start_authentication(&mut connection, identity)
        .await
        .unwrap();
    options.public_key.challenge = vec![1; 32].into();
    let answer = device.do_authentication(origin(), options).unwrap();
    error(
        service
            .finish_authentication(
                &mut connection,
                identity,
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );
    let (token, options) = service
        .start_authentication(&mut connection, identity)
        .await
        .unwrap();
    let mut options = serde_json::to_value(options).unwrap();
    options["publicKey"]["userVerification"] = "discouraged".into();
    let answer = device
        .do_authentication(origin(), serde_json::from_value(options).unwrap())
        .unwrap();
    error(
        service
            .finish_authentication(
                &mut connection,
                identity,
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );

    // Registration also rejects a signed response lacking UV and a different origin.
    let (token, options) = service
        .start_registration(&mut connection, id(base + 3))
        .await
        .unwrap();
    let mut options = serde_json::to_value(options).unwrap();
    options["publicKey"]["authenticatorSelection"]["userVerification"] = "discouraged".into();
    let answer = auth()
        .do_registration(origin(), serde_json::from_value(options).unwrap())
        .unwrap();
    error(
        service
            .finish_registration(
                &mut connection,
                id(base + 3),
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );
    let (token, options) = service
        .start_registration(&mut connection, id(base + 3))
        .await
        .unwrap();
    let answer = auth()
        .do_registration(Url::parse("http://localhost:3001").unwrap(), options)
        .unwrap();
    error(
        service
            .finish_registration(
                &mut connection,
                id(base + 3),
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );

    // Mutations preserve the surrounding JSON but break each authenticated response component.
    for field in ["signature", "authenticatorData", "clientDataJSON"] {
        let (token, options) = service
            .start_authentication(&mut connection, identity)
            .await
            .unwrap();
        let mut answer = device.do_authentication(origin(), options).unwrap();
        match field {
            "signature" => {
                let mut bytes = answer.response.signature.as_slice().to_vec();
                bytes[0] ^= 1;
                answer.response.signature = bytes.into();
            }
            "authenticatorData" => {
                let mut bytes = answer.response.authenticator_data.as_slice().to_vec();
                bytes[0] ^= 1;
                answer.response.authenticator_data = bytes.into();
            }
            _ => {
                answer.response.client_data_json = b"{}".to_vec().into();
            }
        }
        error(
            service
                .finish_authentication(
                    &mut connection,
                    identity,
                    &token,
                    &serde_json::to_vec(&answer).unwrap(),
                )
                .await
                .unwrap_err(),
        );
    }
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    error(
        service
            .finish_registration(&mut connection, identity, &token, &response)
            .await
            .unwrap_err(),
    );
    error(
        service
            .finish_authentication(&mut connection, identity, &token, &response)
            .await
            .unwrap_err(),
    );
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    error(
        service
            .finish_authentication(&mut connection, other, &token, &response)
            .await
            .unwrap_err(),
    );
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    let mut answer: PublicKeyCredential = serde_json::from_slice(&response).unwrap();
    answer.response.user_handle = Some(vec![0; 16].into());
    error(
        service
            .finish_authentication(
                &mut connection,
                identity,
                &token,
                &serde_json::to_vec(&answer).unwrap(),
            )
            .await
            .unwrap_err(),
    );

    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    sqlx::query("UPDATE identity_access.webauthn_ceremonies SET created_at = 0, expires_at = 1 WHERE identity_id = $1::text::numeric AND NOT consumed").bind(identity.value().to_string()).execute(&mut connection).await.unwrap();
    error(
        service
            .finish_authentication(&mut connection, identity, &token, &response)
            .await
            .unwrap_err(),
    );

    // A new successful registration invalidates any previously issued authentication snapshot.
    let (old, response) = assertion(&service, &mut connection, identity, &mut device).await;
    registration(&service, &mut connection, identity, &mut auth()).await;
    error(
        service
            .finish_authentication(&mut connection, identity, &old, &response)
            .await
            .unwrap_err(),
    );

    // A malformed or oversized answer consumes its reference before response parsing.
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    error(
        service
            .finish_authentication(&mut connection, identity, &token, &vec![0; 65_537])
            .await
            .unwrap_err(),
    );
    error(
        service
            .finish_authentication(&mut connection, identity, &token, &response)
            .await
            .unwrap_err(),
    );

    // Expiry is checked again after waiting for the account lock, not just at consumption.
    let short = Arc::new(LocalhostWebauthn::new(CeremonyLimits::new(3, 100, 8).unwrap()).unwrap());
    let (token, response) = assertion(&short, &mut connection, identity, &mut device).await;
    let mut expiry_guard = connect("postgres").await;
    let mut expiry_tx = expiry_guard.begin().await.unwrap();
    sqlx::query("SELECT identity_id FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric FOR UPDATE").bind(identity.value().to_string()).execute(&mut *expiry_tx).await.unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let worker = Arc::clone(&short);
    let pending = tokio::spawn(async move {
        worker
            .finish_authentication(&mut reader, identity, &token, &response)
            .await
    });
    waiters(&mut connection, &[reader_pid]).await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let expired: bool = sqlx::query_scalar("SELECT floor(EXTRACT(epoch FROM clock_timestamp())) >= min(expires_at) FROM identity_access.webauthn_ceremonies WHERE identity_id = $1::text::numeric AND consumed AND expires_at > created_at + 1")
                .bind(identity.value().to_string()).fetch_one(&mut connection).await.unwrap();
            if expired { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.unwrap();
    expiry_tx.commit().await.unwrap();
    error(pending.await.unwrap().unwrap_err());

    // Cancellation after durable consumption cannot restore the ceremony.
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    let retry = duplicate(&token);
    let mut cancellation_guard = connect("postgres").await;
    let mut cancellation_tx = cancellation_guard.begin().await.unwrap();
    sqlx::query("SELECT identity_id FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric FOR UPDATE").bind(identity.value().to_string()).execute(&mut *cancellation_tx).await.unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let worker = Arc::clone(&service);
    let answer = response.clone();
    let pending = tokio::spawn(async move {
        worker
            .finish_authentication(&mut reader, identity, &token, &answer)
            .await
    });
    waiters(&mut connection, &[reader_pid]).await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    cancellation_tx.commit().await.unwrap();
    error(
        service
            .finish_authentication(&mut connection, identity, &retry, &response)
            .await
            .unwrap_err(),
    );

    // Two actual row waiters compete for the same single-use ceremony.
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    let mut guard = connect("postgres").await;
    let mut tx = guard.begin().await.unwrap();
    sqlx::query("SELECT fingerprint FROM identity_access.webauthn_ceremonies WHERE identity_id = $1::text::numeric AND NOT consumed FOR UPDATE").bind(identity.value().to_string()).execute(&mut *tx).await.unwrap();
    let mut a = connect("postgres").await;
    let mut b = connect("postgres").await;
    let pids = [pid(&mut a).await, pid(&mut b).await];
    let worker = Arc::clone(&service);
    let token_a = duplicate(&token);
    let answer = response.clone();
    let a = tokio::spawn(async move {
        worker
            .finish_authentication(&mut a, identity, &token_a, &answer)
            .await
    });
    let worker = Arc::clone(&service);
    let token_b = duplicate(&token);
    let b = tokio::spawn(async move {
        worker
            .finish_authentication(&mut b, identity, &token_b, &response)
            .await
    });
    waiters(&mut connection, &pids).await;
    tx.commit().await.unwrap();
    let (a, b) = (a.await.unwrap(), b.await.unwrap());
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);

    // A concurrent revocation wins at the account lock; the old assertion cannot succeed.
    let (token, response) = assertion(&service, &mut connection, identity, &mut device).await;
    let mut tx = guard.begin().await.unwrap();
    sqlx::query("SELECT identity_id FROM identity_access.webauthn_accounts WHERE identity_id = $1::text::numeric FOR UPDATE").bind(identity.value().to_string()).execute(&mut *tx).await.unwrap();
    let mut reader = connect("postgres").await;
    let reader_pid = pid(&mut reader).await;
    let worker = Arc::clone(&service);
    let pending = tokio::spawn(async move {
        worker
            .finish_authentication(&mut reader, identity, &token, &response)
            .await
    });
    waiters(&mut connection, &[reader_pid]).await;
    sqlx::query("DELETE FROM identity_access.webauthn_credentials WHERE credential_id = $1")
        .bind(key.expose())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE identity_access.webauthn_accounts SET revision = $2 WHERE identity_id = $1::text::numeric").bind(identity.value().to_string()).bind(vec![7_u8; 32]).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    error(pending.await.unwrap().unwrap_err());

    // Limits and negative RLS paths use existing positive fixtures.
    let limited = LocalhostWebauthn::new(CeremonyLimits::new(300, 100, 1).unwrap()).unwrap();
    error(
        limited
            .start_registration(&mut connection, other)
            .await
            .unwrap_err(),
    );
    let pending_limit = LocalhostWebauthn::new(CeremonyLimits::new(300, 1, 8).unwrap()).unwrap();
    error(
        pending_limit
            .start_registration(&mut connection, id(base + 5))
            .await
            .unwrap_err(),
    );
    let mut transaction = connection.begin().await.unwrap();
    error(
        service
            .start_registration(&mut transaction, identity)
            .await
            .unwrap_err(),
    );
    transaction.rollback().await.unwrap();
    let mut runtime = connect("kc_identity_test_runtime").await;
    error(
        service
            .start_registration(&mut runtime, identity)
            .await
            .unwrap_err(),
    );
    error(
        service
            .start_authentication(&mut runtime, identity)
            .await
            .unwrap_err(),
    );
    error(
        service
            .revoke(&mut runtime, identity, &key)
            .await
            .unwrap_err(),
    );
    sqlx::query("SET ROLE kc_identity_owner")
        .execute(&mut runtime)
        .await
        .unwrap_err();
    for query in [
        "SELECT count(*) FROM identity_access.webauthn_accounts",
        "SELECT count(*) FROM identity_access.webauthn_credentials",
        "SELECT count(*) FROM identity_access.webauthn_ceremonies",
    ] {
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&mut runtime)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
    let mut owner = connect("postgres").await;
    sqlx::query("SET ROLE kc_identity_owner")
        .execute(&mut owner)
        .await
        .unwrap();
    for query in [
        "SELECT count(*) FROM identity_access.webauthn_accounts",
        "SELECT count(*) FROM identity_access.webauthn_credentials",
        "SELECT count(*) FROM identity_access.webauthn_ceremonies",
    ] {
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&mut owner)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
    assert!(consumed(&mut connection, identity).await > 0);
}
