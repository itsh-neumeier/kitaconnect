//! Actual TCP/mTLS/HTTP; all certificates and synthetic request bytes stay in RAM.
use kitaconnect_service_transport::{
    Client, RegisteredDnsName, Server, VerifiedConnection,
    oauth_http::{ClientLimits, OAuthEndpoint, OAuthHttpClient},
};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertifiedIssuer, ExtendedKeyUsagePurpose,
    IsCa, KeyPair, KeyUsagePurpose,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::oneshot,
    time::{sleep, timeout},
};

struct Leaf {
    cert: Certificate,
    key: KeyPair,
}
impl Leaf {
    fn chain(&self) -> Vec<CertificateDer<'static>> {
        vec![self.cert.der().clone()]
    }
    fn key(&self) -> PrivateKeyDer<'static> {
        PrivatePkcs8KeyDer::from(self.key.serialize_der()).into()
    }
}
struct Fixture {
    ca: CertifiedIssuer<'static, KeyPair>,
    server: Leaf,
    client: Leaf,
}
impl Fixture {
    fn new() -> Self {
        let mut p = CertificateParams::new(Vec::<String>::new()).unwrap();
        p.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        p.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca = CertifiedIssuer::self_signed(p, KeyPair::generate().unwrap()).unwrap();
        let make = |name: &str, usage| {
            let mut p = CertificateParams::new(vec![name.to_owned()]).unwrap();
            p.not_before = OffsetDateTime::now_utc() - time::Duration::minutes(1);
            p.not_after = OffsetDateTime::now_utc() + time::Duration::hours(1);
            p.key_usages = vec![KeyUsagePurpose::DigitalSignature];
            p.extended_key_usages = vec![usage];
            let key = KeyPair::generate().unwrap();
            let cert = p.signed_by(&key, &ca).unwrap();
            Leaf { cert, key }
        };
        let server = make("identity.test", ExtendedKeyUsagePurpose::ServerAuth);
        let client = make("gateway.test", ExtendedKeyUsagePurpose::ClientAuth);
        Self { ca, server, client }
    }
    fn server(&self) -> Server {
        Server::new(
            self.server.chain(),
            self.server.key(),
            vec![self.ca.der().clone()],
        )
        .unwrap()
    }
    fn client(&self, limits: ClientLimits) -> OAuthHttpClient {
        OAuthHttpClient::new(
            Client::new(
                self.client.chain(),
                self.client.key(),
                vec![self.ca.der().clone()],
            )
            .unwrap(),
            RegisteredDnsName::new("identity.test").unwrap(),
            limits,
        )
        .unwrap()
    }
}
fn limits() -> ClientLimits {
    // Isolated fixture values only; no productive rate or timeout choice.
    ClientLimits {
        parallel: 1,
        form_bytes: 256,
        response_bytes: 256,
        headers: 16,
        header_bytes: 8192,
        handshake_time: Duration::from_secs(2),
        header_time: Duration::from_secs(2),
        body_time: Duration::from_secs(2),
        total_time: Duration::from_secs(3),
    }
}
async fn sockets() -> (TcpStream, TcpStream) {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (c, s) = tokio::join!(TcpStream::connect(l.local_addr().unwrap()), l.accept());
    (c.unwrap(), s.unwrap().0)
}
async fn read_request(stream: VerifiedConnection, path: &str) -> BufReader<VerifiedConnection> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    assert!(line == format!("POST {path} HTTP/1.1\r\n"));
    let mut length = 0;
    let mut bytes = 0;
    let mut form = false;
    loop {
        line.clear();
        assert!(reader.read_line(&mut line).await.unwrap() > 0);
        bytes += line.len();
        assert!(bytes < 8192);
        if line == "\r\n" {
            break;
        }
        let line = line.to_ascii_lowercase();
        if let Some(l) = line.strip_prefix("content-length:") {
            length = l.trim().parse().unwrap();
        }
        if line == "content-type: application/x-www-form-urlencoded\r\n" {
            form = true;
        }
        assert!(!line.starts_with("authorization:") && !line.starts_with("cookie:"));
    }
    assert!(form && length <= 256);
    let mut body = zeroize::Zeroizing::new(vec![0; length]);
    reader.read_exact(&mut body).await.unwrap();
    assert!(body.as_slice() == b"client_id=gateway&token=synthetic");
    reader
}
struct ResponseSpec {
    status: u16,
    headers: String,
    body: Vec<u8>,
    header_delay: Duration,
    body_delay: Duration,
    truncate: bool,
}
impl ResponseSpec {
    fn json() -> Self {
        Self {
            status: 200,
            headers:
                "Content-Type: application/json\r\nCache-Control: no-store\r\nPragma: no-cache\r\n"
                    .into(),
            body: b"{\"active\":false}".to_vec(),
            header_delay: Duration::ZERO,
            body_delay: Duration::ZERO,
            truncate: false,
        }
    }
}
async fn respond(server: Server, socket: TcpStream, path: &'static str, spec: ResponseSpec) {
    let stream = server.accept(socket, Duration::from_secs(2)).await.unwrap();
    let gate = stream.request_peer();
    let mut reader = read_request(stream, path).await;
    sleep(spec.header_delay).await;
    let header = format!(
        "HTTP/1.1 {} fixture\r\nContent-Length: {}\r\n{}\r\n",
        spec.status,
        spec.body.len(),
        spec.headers
    );
    if reader.get_mut().write_all(header.as_bytes()).await.is_ok() {
        sleep(spec.body_delay).await;
        if spec.truncate {
            let _ = reader.get_mut().write_all(&spec.body[..1]).await;
            drop(reader);
            assert!(gate.certificate().is_err());
            return;
        }
        let _ = reader.get_mut().write_all(&spec.body).await;
    }
    // Observe closure without extending the client's transport lifetime or leaving a task.
    let mut byte = [0];
    let result = timeout(Duration::from_secs(2), reader.read(&mut byte))
        .await
        .expect("caller releases actual connection");
    assert!(matches!(result, Ok(0) | Err(_)));
    assert!(gate.certificate().is_err());
}
async fn call(
    f: &Fixture,
    client: &OAuthHttpClient,
    endpoint: OAuthEndpoint,
    path: &'static str,
    spec: ResponseSpec,
) -> Result<
    kitaconnect_service_transport::oauth_http::OAuthReply,
    kitaconnect_service_transport::oauth_http::HttpError,
> {
    let (c, s) = sockets().await;
    let (reply, ()) = tokio::join!(
        client.request(c, endpoint, b"client_id=gateway&token=synthetic"),
        respond(f.server(), s, path, spec)
    );
    reply
}

#[tokio::test]
async fn actual_client_posts_both_paths_and_keeps_secrets_out_of_debug() {
    let f = Fixture::new();
    let client = f.client(limits());
    for (endpoint, path) in [
        (OAuthEndpoint::Token, "/oauth/token"),
        (OAuthEndpoint::Introspect, "/oauth/introspect"),
    ] {
        let reply = call(&f, &client, endpoint, path, ResponseSpec::json())
            .await
            .unwrap();
        assert_eq!(reply.status(), 200);
        assert!(reply.body() == b"{\"active\":false}");
        let expected: [u8; 32] = Sha256::digest(f.server.cert.der().as_ref()).into();
        assert_eq!(reply.server_certificate(), expected);
        assert_eq!(format!("{reply:?}"), "OAuthReply([REDACTED])");
    }
    let mut empty = ResponseSpec::json();
    empty.status = 503;
    empty.headers = "Cache-Control: no-store\r\nPragma: no-cache\r\n".into();
    empty.body.clear();
    let reply = call(&f, &client, OAuthEndpoint::Token, "/oauth/token", empty)
        .await
        .unwrap();
    assert!(reply.status() == 503 && reply.body().is_empty());
}

#[tokio::test]
async fn actual_client_refuses_bad_headers_redirects_and_oversized_data() {
    let f = Fixture::new();
    let client = f.client(limits());
    for kind in 0..10 {
        let mut spec = ResponseSpec::json();
        match kind {
            0 => spec.headers = spec.headers.replace("no-store", "public"),
            1 => spec.headers.push_str("Cache-Control: no-store\r\n"),
            2 => spec.headers = spec.headers.replace("application/json", "text/plain"),
            3 => spec.headers.push_str("Content-Type: application/json\r\n"),
            4 => spec.status = 302,
            5 => spec.body = vec![b'x'; 257],
            6 => spec.headers.push_str(&"X-Test: bounded\r\n".repeat(17)),
            7 => spec
                .headers
                .push_str(&format!("X-Large: {}\r\n", "x".repeat(9000))),
            8 => spec.headers = spec.headers.replace("Pragma: no-cache\r\n", ""),
            9 => spec.truncate = true,
            _ => unreachable!(),
        }
        assert!(
            call(
                &f,
                &client,
                OAuthEndpoint::Introspect,
                "/oauth/introspect",
                spec
            )
            .await
            .is_err(),
            "reject response case {kind}"
        );
    }
}

#[tokio::test]
async fn actual_client_bounds_header_body_total_and_handshake_time() {
    let f = Fixture::new();
    for kind in 0..3 {
        let mut configured = limits();
        let mut spec = ResponseSpec::json();
        match kind {
            0 => {
                configured.header_time = Duration::from_millis(100);
                spec.header_delay = Duration::from_millis(250);
            }
            1 => {
                configured.body_time = Duration::from_millis(100);
                spec.body_delay = Duration::from_millis(250);
            }
            2 => {
                configured.total_time = Duration::from_millis(100);
                spec.body_delay = Duration::from_millis(250);
            }
            _ => unreachable!(),
        }
        assert!(
            call(
                &f,
                &f.client(configured),
                OAuthEndpoint::Token,
                "/oauth/token",
                spec
            )
            .await
            .is_err()
        );
    }
    let mut configured = limits();
    configured.handshake_time = Duration::from_millis(100);
    let client = f.client(configured);
    let (c, _silent) = sockets().await;
    assert!(
        timeout(
            Duration::from_secs(1),
            client.request(c, OAuthEndpoint::Token, b"synthetic")
        )
        .await
        .unwrap()
        .is_err()
    );
    // Outbound body bound is checked before starting TLS on the supplied socket.
    let (c, mut peer) = sockets().await;
    let client = f.client(limits());
    assert!(
        client
            .request(c, OAuthEndpoint::Token, &vec![b'x'; 257])
            .await
            .is_err()
    );
    let mut byte = [0];
    assert_eq!(
        timeout(Duration::from_secs(1), peer.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn actual_client_cancellation_releases_stream_and_shared_budget() {
    let f = Fixture::new();
    let client = Arc::new(f.client(limits()));
    let (c, s) = sockets().await;
    let server = f.server();
    let (started_tx, started) = oneshot::channel();
    let (closed_tx, closed) = oneshot::channel();
    let server_task = tokio::spawn(async move {
        let stream = server.accept(s, Duration::from_secs(2)).await.unwrap();
        let gate = stream.request_peer();
        let mut reader = read_request(stream, "/oauth/introspect").await;
        started_tx.send(()).unwrap();
        let mut byte = [0];
        let result = reader.read(&mut byte).await;
        assert!(matches!(result, Ok(0) | Err(_)));
        assert!(gate.certificate().is_err());
        closed_tx.send(()).unwrap();
    });
    let shared = client.clone();
    let request = tokio::spawn(async move {
        shared
            .request(
                c,
                OAuthEndpoint::Introspect,
                b"client_id=gateway&token=synthetic",
            )
            .await
    });
    timeout(Duration::from_secs(1), started)
        .await
        .unwrap()
        .unwrap();
    let (extra, mut rejected) = sockets().await;
    assert!(
        client
            .request(extra, OAuthEndpoint::Token, b"synthetic")
            .await
            .is_err()
    );
    let mut byte = [0];
    assert_eq!(
        timeout(Duration::from_secs(1), rejected.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    request.abort();
    assert!(request.await.unwrap_err().is_cancelled());
    timeout(Duration::from_secs(1), closed)
        .await
        .expect("aborted caller drops real TLS stream")
        .unwrap();
    server_task.await.unwrap();
    assert!(
        call(
            &f,
            &client,
            OAuthEndpoint::Token,
            "/oauth/token",
            ResponseSpec::json()
        )
        .await
        .is_ok()
    );
}
