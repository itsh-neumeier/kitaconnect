//! Actual HTTP/mTLS/PostgreSQL fixture only. Synthetic administration is not runtime permission.
use kitaconnect_identity_access::{
    service_auth::RegisteredNames,
    service_http::{Limits, OAuthServer},
    service_tokens::{AuthorityConfiguration, ServiceName},
};
use kitaconnect_service_oauth::{
    ExpectedActive, IntrospectionResponse, ResponseLimits, TokenResponse,
};
use kitaconnect_service_transport::oauth_http::{
    ClientLimits, OAuthEndpoint, OAuthHttpClient, OAuthReply,
};
use kitaconnect_service_transport::{Client, RegisteredDnsName, Server, VerifiedConnection};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::{sleep, timeout},
};

const FORM: &str = "Content-Type: application/x-www-form-urlencoded\r\n";
const TOKEN: &str = "client_id=gateway&grant_type=client_credentials&scope=session%3Acheck";
#[path = "support/authenticated_membership.rs"]
mod authenticated_membership;
#[path = "support/mod.rs"]
mod database_witness;
fn options(user: &str) -> PgConnectOptions {
    PgConnectOptions::new()
        .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
        .database("kc_service_http_test")
        .username(user)
        .ssl_mode(PgSslMode::Disable)
        .application_name("kc_oauth_http_request")
}
fn config() -> AuthorityConfiguration {
    AuthorityConfiguration::new([11; 16], "https://identity.test/").unwrap()
}
fn limits() -> Limits {
    // Explicit isolated fixture values, not production recommendations.
    Limits {
        connections: 8,
        requests: 1,
        body_bytes: 512,
        parameters: 8,
        headers: 16,
        header_bytes: 8192,
        handshake_time: Duration::from_secs(2),
        header_time: Duration::from_secs(10),
        request_time: Duration::from_secs(2),
        connection_time: Duration::from_secs(45),
    }
}
struct Leaf {
    chain: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    digest: [u8; 32],
    start: i64,
    end: i64,
}
fn leaf(ca: &CertifiedIssuer<'_, KeyPair>, name: &str) -> Leaf {
    let mut p = CertificateParams::new(vec![name.to_owned()]).unwrap();
    let now = OffsetDateTime::now_utc().unix_timestamp();
    p.not_before = OffsetDateTime::from_unix_timestamp(now - 60).unwrap();
    p.not_after = OffsetDateTime::from_unix_timestamp(now + 3600).unwrap();
    p.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    p.extended_key_usages = vec![
        ExtendedKeyUsagePurpose::ClientAuth,
        ExtendedKeyUsagePurpose::ServerAuth,
    ];
    let key = KeyPair::generate().unwrap();
    let cert = p.signed_by(&key, ca).unwrap();
    Leaf {
        digest: Sha256::digest(cert.der().as_ref()).into(),
        chain: vec![cert.der().clone()],
        key: PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
        start: now - 60,
        end: now + 3600,
    }
}
fn server(
    identity: &Leaf,
    ca: &CertifiedIssuer<'_, KeyPair>,
    user: &str,
    limits: Limits,
) -> Arc<OAuthServer> {
    Arc::new(
        OAuthServer::new(
            Server::new(
                identity.chain.clone(),
                identity.key.clone_key(),
                vec![ca.der().clone()],
            )
            .unwrap(),
            config(),
            RegisteredNames::new(vec![
                (
                    ServiceName::IdentityAccess,
                    RegisteredDnsName::new("identity.test").unwrap(),
                ),
                (
                    ServiceName::OrganizationConfig,
                    RegisteredDnsName::new("organization.test").unwrap(),
                ),
                (
                    ServiceName::Gateway,
                    RegisteredDnsName::new("gateway.test").unwrap(),
                ),
            ])
            .unwrap(),
            options(user),
            limits,
            120,
        )
        .unwrap(),
    )
}
struct Wire(
    BufReader<VerifiedConnection>,
    JoinHandle<Result<(), kitaconnect_identity_access::service_http::HttpError>>,
);
impl Drop for Wire {
    fn drop(&mut self) {
        self.1.abort();
    }
}
async fn wire(server: Arc<OAuthServer>, leaf: &Leaf, ca: &CertifiedIssuer<'_, KeyPair>) -> Wire {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (accepted, _) = listener.accept().await.unwrap();
    let task = tokio::spawn(server.serve(accepted));
    let stream = Client::new(
        leaf.chain.clone(),
        leaf.key.clone_key(),
        vec![ca.der().clone()],
    )
    .unwrap()
    .connect(
        socket,
        ServerName::try_from("identity.test").unwrap(),
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    Wire(BufReader::new(stream), task)
}

async fn library_request(
    server: Arc<OAuthServer>,
    leaf: &Leaf,
    ca: &CertifiedIssuer<'_, KeyPair>,
    endpoint: OAuthEndpoint,
    form: &[u8],
) -> OAuthReply {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (accepted, _) = listener.accept().await.unwrap();
    let task = tokio::spawn(server.serve(accepted));
    let client = OAuthHttpClient::new(
        Client::new(
            leaf.chain.clone(),
            leaf.key.clone_key(),
            vec![ca.der().clone()],
        )
        .unwrap(),
        RegisteredDnsName::new("identity.test").unwrap(),
        ClientLimits {
            parallel: 1,
            form_bytes: 512,
            response_bytes: 2048,
            headers: 16,
            header_bytes: 8192,
            handshake_time: Duration::from_secs(2),
            header_time: Duration::from_secs(2),
            body_time: Duration::from_secs(2),
            total_time: Duration::from_secs(3),
        },
    )
    .unwrap();
    let reply = client.request(socket, endpoint, form).await.unwrap();
    // The client owns and drops this connection; TLS EOF can be reported as a transport error.
    let _ = timeout(Duration::from_secs(2), task)
        .await
        .expect("actual server connection released")
        .unwrap();
    reply
}
struct Reply {
    status: u16,
    headers: String,
    body: Vec<u8>,
}
impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("expected bounded JSON response")
    }
    fn code(&self, status: u16, code: &str) {
        assert_eq!(self.status, status);
        assert!(self.json() == json!({"error":code})); // Never dump raw tokens in assertion output.
    }
}
impl Wire {
    async fn send(&mut self, method: &str, path: &str, headers: &str, body: &str) {
        let mut request = zeroize::Zeroizing::new(format!(
            "{method} {path} HTTP/1.1\r\nHost: identity.test\r\nContent-Length: {}\r\n{headers}\r\n",
            body.len()
        ));
        request.push_str(body);
        self.0
            .get_mut()
            .write_all(request.as_bytes())
            .await
            .expect("write bounded synthetic HTTP request without logging credentials");
        self.0.get_mut().flush().await.unwrap();
    }
    async fn receive(&mut self) -> Reply {
        timeout(Duration::from_secs(5), async {
            let mut line = String::new();
            self.0.read_line(&mut line).await.unwrap();
            let status = line
                .split_whitespace()
                .nth(1)
                .expect("HTTP status")
                .parse()
                .unwrap();
            let mut headers = String::new();
            let mut length = 0;
            loop {
                line.clear();
                assert!(self.0.read_line(&mut line).await.unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().unwrap();
                }
                assert!(
                    !lower.starts_with("transfer-encoding:"),
                    "fixture expects bounded known-length bodies"
                );
                headers.push_str(&lower);
            }
            assert!(length <= 2048);
            let mut body = vec![0; length];
            self.0.read_exact(&mut body).await.unwrap();
            assert!(headers.contains("cache-control: no-store\r\n"));
            assert!(headers.contains("pragma: no-cache\r\n"));
            Reply {
                status,
                headers,
                body,
            }
        })
        .await
        .expect("bounded actual HTTP response")
    }
    async fn post(&mut self, path: &str, body: &str) -> Reply {
        self.send("POST", path, FORM, body).await;
        self.receive().await
    }
}
async fn register(c: &mut PgConnection, name: &str, leaf: &Leaf) {
    sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,$2,$3::text::numeric,$4::text::numeric,false)")
        .bind(leaf.digest.as_slice()).bind(name).bind(leaf.start.to_string()).bind(leaf.end.to_string()).execute(c).await.unwrap();
}
async fn grant_state(c: &mut PgConnection, active: bool) {
    sqlx::query("UPDATE identity_access.service_grants SET active=$1,version=version+1 WHERE client='gateway' AND scope='session:check'")
        .bind(active).execute(c).await.unwrap();
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn actual_http_is_bound_current_and_bounded() {
    let mut admin =
        PgConnection::connect_with(&options("postgres").application_name("kc_http_fixture"))
            .await
            .unwrap();
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM identity_access.service_tokens")
            .fetch_one(&mut admin)
            .await
            .unwrap();
        assert!(count > 0, "confirmed token hashes survive restart");
    } else {
        sqlx::query("INSERT INTO identity_access.service_authority VALUES(true,$1,$2)")
            .bind(config().installation().as_slice())
            .bind(config().issuer())
            .execute(&mut admin)
            .await
            .unwrap();
        sqlx::raw_sql("INSERT INTO identity_access.service_clients VALUES('identity-access',1,true),('organization-config',1,true),('gateway',1,true); INSERT INTO identity_access.service_grants VALUES('gateway','identity-access','session:check',1,true),('identity-access','organization-config','membership:read',1,true)")
            .execute(&mut admin).await.unwrap();
    }
    let mut p = CertificateParams::new(Vec::<String>::new()).unwrap();
    p.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    p.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let ca = CertifiedIssuer::self_signed(p, KeyPair::generate().unwrap()).unwrap();
    let identity = leaf(&ca, "identity.test");
    let gateway = leaf(&ca, "gateway.test");
    let organization = leaf(&ca, "organization.test");
    for (name, leaf) in [
        ("identity-access", &identity),
        ("gateway", &gateway),
        ("organization-config", &organization),
    ] {
        register(&mut admin, name, leaf).await;
    }
    let http = server(&identity, &ca, "postgres", limits());
    authenticated_membership::check(&http, &identity, &organization, &gateway, &ca, after).await;
    let mut g = wire(http.clone(), &gateway, &ca).await;
    let mut i = wire(http.clone(), &identity, &ca).await;
    let mut o = wire(http.clone(), &organization, &ca).await;
    // Join the actual library client, actual Axum server and PostgreSQL registration/token state.
    let library_token = library_request(
        http.clone(),
        &gateway,
        &ca,
        OAuthEndpoint::Token,
        TOKEN.as_bytes(),
    )
    .await;
    assert_eq!(library_token.status(), 200);
    assert_eq!(library_token.server_certificate(), identity.digest);
    let parsed_token = TokenResponse::parse(
        library_token.status(),
        library_token.body(),
        "session:check",
        ResponseLimits::new(2048).unwrap(),
    )
    .unwrap();
    assert!((1..=120).contains(&parsed_token.expires_in()));
    let encoded = parsed_token.expose_token();
    let library_form =
        zeroize::Zeroizing::new(format!("client_id=identity-access&token={encoded}"));
    let library_active = library_request(
        http.clone(),
        &identity,
        &ca,
        OAuthEndpoint::Introspect,
        library_form.as_bytes(),
    )
    .await;
    assert_eq!(library_active.status(), 200);
    let fresh: i64 =
        sqlx::query_scalar("SELECT floor(extract(epoch FROM clock_timestamp()))::bigint")
            .fetch_one(&mut admin)
            .await
            .unwrap();
    let expected = ExpectedActive {
        issuer: "https://identity.test/",
        client: "gateway",
        target: "identity-access",
        scope: "session:check",
        certificate: gateway.digest,
        now: u64::try_from(fresh).unwrap(),
    };
    assert!(matches!(
        IntrospectionResponse::parse(
            library_active.status(),
            library_active.body(),
            expected,
            ResponseLimits::new(2048).unwrap()
        )
        .unwrap(),
        IntrospectionResponse::Active(_)
    ));
    let library_value: Value = serde_json::from_slice(library_active.body()).unwrap();
    assert!(
        library_value["active"] == true
            && library_value["client_id"] == "gateway"
            && library_value["aud"] == "identity-access"
            && library_value["iss"] == config().issuer()
    );
    let response = g.post("/oauth/token", TOKEN).await;
    assert_eq!(response.status, 200);
    let value = response.json();
    assert!(
        value["expires_in"]
            .as_u64()
            .is_some_and(|n| (1..=120).contains(&n))
    );
    assert!(value["scope"] == "session:check" && value["token_type"] == "Bearer");
    let token = value["access_token"].as_str().unwrap();
    assert_eq!(token.len(), 43);
    let request = format!("client_id=identity-access&token={token}");
    let active = i.post("/oauth/introspect", &request).await;
    assert_eq!(active.status, 200);
    let result = active.json();
    assert!(
        result["active"] == true
            && result["client_id"] == "gateway"
            && result["aud"] == "identity-access"
    );
    assert!(result["iss"] == config().issuer());
    use base64::Engine as _;
    assert!(
        library_value["cnf"]["x5t#S256"]
            == base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(gateway.digest)
    );
    assert!(
        result["cnf"]["x5t#S256"]
            == base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(gateway.digest)
    );
    let foreign = o
        .post(
            "/oauth/introspect",
            &format!("client_id=organization-config&token={token}"),
        )
        .await;
    assert!(foreign.status == 200 && foreign.json() == json!({"active":false}));
    assert!(
        i.post("/oauth/introspect", "client_id=identity-access&token=bad")
            .await
            .json()
            == json!({"active":false})
    );
    g.post("/oauth/introspect", "client_id=gateway&token=bad")
        .await
        .code(401, "invalid_client");
    // Claimed identity and forged proxy certificate headers never replace the actual peer.
    g.send(
        "POST",
        "/oauth/token",
        &format!("{FORM}X-Client-Cert: identity.test\r\n"),
        "client_id=identity-access&grant_type=client_credentials&scope=membership:read",
    )
    .await;
    g.receive().await.code(400, "invalid_client");
    g.send("GET", "/oauth/token", "", "").await;
    let response = g.receive().await;
    assert_eq!(response.status, 405);
    assert!(response.headers.contains("allow: post\r\n"));
    for (path, headers, body, status) in [
        ("/unknown", FORM, TOKEN, 404),
        (
            "/oauth/token",
            "Content-Type: application/json\r\n",
            TOKEN,
            415,
        ),
        (
            "/oauth/token",
            "Content-Type: application/x-www-form-urlencoded\r\nContent-Type: application/json\r\n",
            TOKEN,
            415,
        ),
        ("/oauth/token?client_id=gateway", FORM, TOKEN, 400),
        (
            "/oauth/token",
            "Content-Type: application/x-www-form-urlencoded\r\nCookie: session=untrusted\r\n",
            TOKEN,
            400,
        ),
        (
            "/oauth/token",
            "Authorization: Bearer untrusted\r\n",
            TOKEN,
            401,
        ),
        (
            "/oauth/token",
            "Authorization: Basic untrusted\r\nAuthorization: Bearer untrusted\r\n",
            TOKEN,
            400,
        ),
    ] {
        // Early rejections can close a connection with an unread request body.
        let mut rejected = wire(http.clone(), &gateway, &ca).await;
        rejected.send("POST", path, headers, body).await;
        let reply = rejected.receive().await;
        assert_eq!(reply.status, status);
        if status == 401 {
            assert!(
                reply
                    .headers
                    .contains("www-authenticate: bearer realm=\"service\"\r\n")
            );
        }
    }
    let mut large = wire(http.clone(), &gateway, &ca).await;
    large
        .send("POST", "/oauth/token", FORM, &"x".repeat(513))
        .await;
    assert_eq!(large.receive().await.status, 413);
    drop(large);
    g.post(
        "/oauth/token",
        "client_id=gateway&grant_type=client_credentials&scope=membership:read",
    )
    .await
    .code(400, "invalid_scope");
    grant_state(&mut admin, false).await;
    g.post("/oauth/token", TOKEN)
        .await
        .code(400, "invalid_scope");
    assert!(i.post("/oauth/introspect", &request).await.json() == json!({"active":false}));
    grant_state(&mut admin, true).await;
    // Old token remains terminal even when the grant is enabled again.
    assert!(i.post("/oauth/introspect", &request).await.json() == json!({"active":false}));
    assert_eq!(g.post("/oauth/token", TOKEN).await.status, 200);
    sqlx::query("UPDATE identity_access.service_clients SET active=false,version=version+1 WHERE name='gateway'").execute(&mut admin).await.unwrap();
    g.post("/oauth/token", TOKEN)
        .await
        .code(400, "invalid_client");
    sqlx::query("UPDATE identity_access.service_clients SET active=true,version=version+1 WHERE name='gateway'").execute(&mut admin).await.unwrap();
    assert_eq!(g.post("/oauth/token", TOKEN).await.status, 200);
    // Trusted CA + right SAN without exact registered fingerprint is insufficient.
    let unregistered = leaf(&ca, "gateway.test");
    let mut unknown = wire(http.clone(), &unregistered, &ca).await;
    unknown
        .post("/oauth/token", TOKEN)
        .await
        .code(400, "invalid_client");
    drop(unknown);
    // Runtime RLS stays closed over actual HTTP as well; no allowing policy is introduced.
    let closed = server_for_runtime(&identity, &ca);
    let mut runtime = wire(closed, &gateway, &ca).await;
    runtime
        .post("/oauth/token", TOKEN)
        .await
        .code(400, "invalid_client");
    drop(runtime);

    // Block the actual registration query. A separate witness observes its lock and cleanup.
    let mut blocker = admin.begin().await.unwrap();
    sqlx::raw_sql("LOCK TABLE identity_access.service_clients IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await
        .unwrap();
    g.send("POST", "/oauth/token", FORM, TOKEN).await;
    let mut witness =
        PgConnection::connect_with(&options("postgres").application_name("kc_http_witness"))
            .await
            .unwrap();
    let blocked:i32=timeout(Duration::from_secs(1),async {
        loop {
            let pid:Option<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='kc_oauth_http_request' AND wait_event_type='Lock' LIMIT 1")
                .fetch_optional(&mut witness).await.unwrap();
            if let Some(pid)=pid { break pid; } sleep(Duration::from_millis(10)).await;
        }
    }).await.expect("request actually waits on database lock");
    o.send(
        "POST",
        "/oauth/introspect",
        FORM,
        "client_id=organization-config&token=bad",
    )
    .await;
    let reply = o.receive().await;
    assert!(reply.status == 429 && reply.body.is_empty());
    let reply = g.receive().await;
    assert!(reply.status == 503 && reply.body.is_empty());
    timeout(Duration::from_secs(2), async {
        loop {
            let gone: bool =
                sqlx::query_scalar("SELECT NOT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1)")
                    .bind(blocked)
                    .fetch_one(&mut witness)
                    .await
                    .unwrap();
            if gone {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelled physical connection and transaction are gone");
    blocker.rollback().await.unwrap();
    assert_eq!(g.post("/oauth/token", TOKEN).await.status, 200);

    // The request deadline includes a stalled body, before any database connection exists.
    let mut stalled = wire(http.clone(), &organization, &ca).await;
    stalled.0.get_mut().write_all(b"POST /oauth/token HTTP/1.1\r\nHost: identity.test\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 100\r\n\r\nclient_id=").await.unwrap();
    let reply = stalled.receive().await;
    assert!(reply.status == 503 && reply.body.is_empty());
    drop(stalled);

    // A separate one-connection budget rejects a second socket before TLS starts.
    let mut one_limits = limits();
    one_limits.connections = 1;
    let one = server(&identity, &ca, "postgres", one_limits);
    let held = wire(one.clone(), &organization, &ca).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut extra = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (accepted, _) = listener.accept().await.unwrap();
    assert!(
        timeout(Duration::from_millis(100), one.serve(accepted))
            .await
            .expect("budget rejects before handshake wait")
            .is_err()
    );
    let mut byte = [0];
    assert_eq!(
        timeout(Duration::from_secs(1), extra.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    drop(held);

    // The entire connection expires even when a client sends no request at all.
    let mut short_limits = limits();
    short_limits.connection_time = Duration::from_millis(250);
    let short = server(&identity, &ca, "postgres", short_limits);
    let mut idle = wire(short, &organization, &ca).await;
    let result = timeout(Duration::from_secs(1), idle.0.read(&mut byte))
        .await
        .expect("whole connection is bounded");
    assert!(matches!(result, Ok(0) | Err(_)));

    // Header count is enforced before the router and database; no client data is echoed.
    let mut header_limit = wire(http.clone(), &organization, &ca).await;
    let head = format!(
        "POST /oauth/token HTTP/1.1\r\nHost: identity.test\r\n{}\r\n",
        "X-Test: bounded\r\n".repeat(17)
    );
    header_limit
        .0
        .get_mut()
        .write_all(head.as_bytes())
        .await
        .unwrap();
    let mut raw = Vec::new();
    let _ = timeout(Duration::from_secs(1), header_limit.0.read_to_end(&mut raw))
        .await
        .expect("header count closes connection");
    assert!(raw.is_empty() || raw.starts_with(b"HTTP/1.1 431"));

    // Current certificate revocation takes effect on the same established HTTP connection.
    sqlx::query(
        "UPDATE identity_access.service_certificates SET revoked=true WHERE fingerprint=$1",
    )
    .bind(gateway.digest.as_slice())
    .execute(&mut admin)
    .await
    .unwrap();
    g.post("/oauth/token", TOKEN)
        .await
        .code(400, "invalid_client");
    // Malformed token must still authenticate the actual introspection caller first.
    sqlx::query(
        "UPDATE identity_access.service_certificates SET revoked=true WHERE fingerprint=$1",
    )
    .bind(identity.digest.as_slice())
    .execute(&mut admin)
    .await
    .unwrap();
    i.post("/oauth/introspect", "client_id=identity-access&token=bad")
        .await
        .code(401, "invalid_client");

    let mut bad_options = options("postgres");
    bad_options = bad_options.database("kc_http_database_does_not_exist");
    let failing = Arc::new(
        OAuthServer::new(
            Server::new(
                identity.chain.clone(),
                identity.key.clone_key(),
                vec![ca.der().clone()],
            )
            .unwrap(),
            config(),
            RegisteredNames::new(vec![(
                ServiceName::Gateway,
                RegisteredDnsName::new("gateway.test").unwrap(),
            )])
            .unwrap(),
            bad_options,
            limits(),
            120,
        )
        .unwrap(),
    );
    let mut failed = wire(failing, &gateway, &ca).await;
    let reply = failed.post("/oauth/token", TOKEN).await;
    assert!(reply.status == 503 && reply.body.is_empty());
    drop(failed);
    // Header timeout closes the actual TLS connection rather than allocating a body or DB.
    let mut slow_limits = limits();
    slow_limits.header_time = Duration::from_millis(500);
    let slow_server = server(&identity, &ca, "postgres", slow_limits);
    let mut slow = wire(slow_server, &organization, &ca).await;
    slow.0
        .get_mut()
        .write_all(b"POST /oauth/token HTTP/1.1\r\nHost:")
        .await
        .unwrap();
    sleep(Duration::from_millis(650)).await;
    let mut bytes = Vec::new();
    let _ = timeout(Duration::from_secs(1), slow.0.read_to_end(&mut bytes))
        .await
        .expect("header timeout closes socket");
    assert!(bytes.is_empty() || bytes.starts_with(b"HTTP/1.1 408"));
}
fn server_for_runtime(identity: &Leaf, ca: &CertifiedIssuer<'_, KeyPair>) -> Arc<OAuthServer> {
    server(identity, ca, "kc_service_http_runtime", limits())
}
