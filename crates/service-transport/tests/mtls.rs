use kitaconnect_service_transport::{Client, Server, VerifiedConnection};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertifiedIssuer, ExtendedKeyUsagePurpose,
    IsCa, KeyPair, KeyUsagePurpose,
};
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsConnector;

// Fresh, isolated RAM-only CA and keys. Never output or persist their DER/PEM.
fn ca() -> CertifiedIssuer<'static, KeyPair> {
    let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    CertifiedIssuer::self_signed(params, KeyPair::generate().unwrap()).unwrap()
}
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
    fn fingerprint(&self) -> [u8; 32] {
        Sha256::digest(self.cert.der().as_ref()).into()
    }
}
fn leaf(ca: &CertifiedIssuer<'_, KeyPair>, server: bool, start: i64, end: i64) -> Leaf {
    let mut params = CertificateParams::new(vec![if server {
        "organization.test".into()
    } else {
        "identity.test".into()
    }])
    .unwrap();
    let now = OffsetDateTime::now_utc();
    params.not_before = now + time::Duration::seconds(start);
    params.not_after = now + time::Duration::seconds(end);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![if server {
        ExtendedKeyUsagePurpose::ServerAuth
    } else {
        ExtendedKeyUsagePurpose::ClientAuth
    }];
    let key = KeyPair::generate().unwrap();
    let cert = params.signed_by(&key, ca).unwrap();
    Leaf { cert, key }
}
fn server(leaf: &Leaf, ca: &CertifiedIssuer<'_, KeyPair>) -> Server {
    Server::new(leaf.chain(), leaf.key(), vec![ca.der().clone()]).unwrap()
}
fn client(leaf: &Leaf, ca: &CertifiedIssuer<'_, KeyPair>) -> Client {
    Client::new(leaf.chain(), leaf.key(), vec![ca.der().clone()]).unwrap()
}
fn name() -> ServerName<'static> {
    ServerName::try_from("organization.test").unwrap()
}
async fn sockets() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (client, server) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept()
    );
    (client.unwrap(), server.unwrap().0)
}
async fn pair(server: &Server, client: &Client) -> (VerifiedConnection, VerifiedConnection) {
    let (c, s) = sockets().await;
    let (s, c) = tokio::join!(
        server.accept(s, Duration::from_secs(2)),
        client.connect(c, name(), Duration::from_secs(2))
    );
    (s.unwrap(), c.unwrap())
}

#[tokio::test]
async fn actual_mtls_binds_both_certificates_and_rotation() {
    let ca = ca();
    let s = leaf(&ca, true, -60, 3600);
    let c = leaf(&ca, false, -60, 3600);
    let server = server(&s, &ca);
    let client = client(&c, &ca);
    let (mut received, mut sent) = pair(&server, &client).await;
    assert_eq!(
        received.peer_certificate().unwrap().fingerprint(),
        c.fingerprint()
    );
    assert_eq!(
        sent.peer_certificate().unwrap().fingerprint(),
        s.fingerprint()
    );
    let mut bytes = [0; 4];
    sent.write_all(b"test").await.unwrap();
    sent.flush().await.unwrap();
    received.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"test");
    received.write_all(b"pass").await.unwrap();
    received.flush().await.unwrap();
    sent.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"pass");
    let replacement = leaf(&ca, false, -60, 3600);
    let (mut received, _) = pair(&server, &self::client(&replacement, &ca)).await;
    assert_eq!(
        received.peer_certificate().unwrap().fingerprint(),
        replacement.fingerprint()
    );
    assert_ne!(replacement.fingerprint(), c.fingerprint());
}

fn raw_client(leaf: Option<&Leaf>, ca: &CertifiedIssuer<'_, KeyPair>, alpn: bool) -> TlsConnector {
    let mut roots = RootCertStore::empty();
    roots.add(ca.der().clone()).unwrap();
    let builder =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots);
    let mut config = if let Some(leaf) = leaf {
        builder
            .with_client_auth_cert(leaf.chain(), leaf.key())
            .unwrap()
    } else {
        builder.with_no_client_auth()
    };
    if alpn {
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
    }
    TlsConnector::from(Arc::new(config))
}
async fn rejects(
    server: &Server,
    connector: TlsConnector,
    name: ServerName<'static>,
    server_must_reject: bool,
) {
    let (c, s) = sockets().await;
    let (server_result, client_result) = tokio::join!(
        server.accept(s, Duration::from_secs(2)),
        connector.connect(name, c)
    );
    if server_must_reject {
        assert!(server_result.is_err());
    } else {
        assert!(client_result.is_err());
    }
}
#[tokio::test]
async fn actual_mtls_rejects_missing_untrusted_wrong_name_and_invalid_time() {
    let ca = ca();
    let foreign = self::ca();
    let s = leaf(&ca, true, -60, 3600);
    let c = leaf(&ca, false, -60, 3600);
    let server = server(&s, &ca);
    rejects(&server, raw_client(None, &ca, true), name(), true).await;
    let untrusted = leaf(&foreign, false, -60, 3600);
    rejects(
        &server,
        raw_client(Some(&untrusted), &ca, true),
        name(),
        true,
    )
    .await;
    rejects(&server, raw_client(Some(&c), &foreign, true), name(), false).await;
    rejects(
        &server,
        raw_client(Some(&c), &ca, true),
        ServerName::try_from("wrong.test").unwrap(),
        false,
    )
    .await;
    rejects(&server, raw_client(Some(&c), &ca, false), name(), true).await;
    for (start, end) in [(-3600, -60), (60, 3600), (-60, 86401)] {
        let invalid = leaf(&ca, false, start, end);
        assert!(Client::new(invalid.chain(), invalid.key(), vec![ca.der().clone()]).is_err());
        // Raw client bypasses only our local configuration gate, never Rustls verification.
        rejects(&server, raw_client(Some(&invalid), &ca, true), name(), true).await;
        let invalid_server = leaf(&ca, true, start, end);
        assert!(
            Server::new(
                invalid_server.chain(),
                invalid_server.key(),
                vec![ca.der().clone()]
            )
            .is_err()
        );
    }
    assert!(Client::new(c.chain(), c.key(), vec![]).is_err());
    assert!(Server::new(s.chain(), s.key(), vec![]).is_err());
    assert!(Client::new(vec![], c.key(), vec![ca.der().clone()]).is_err());
    assert!(Server::new(s.chain(), c.key(), vec![ca.der().clone()]).is_err());
}

#[tokio::test]
async fn open_connection_rechecks_own_and_peer_expiry() {
    let ca = ca();
    let s = leaf(&ca, true, -60, 3600);
    let c = leaf(&ca, false, -60, 3);
    let (mut received, mut sent) = pair(&server(&s, &ca), &client(&c, &ca)).await;
    assert!(received.peer_certificate().is_ok());
    assert!(sent.peer_certificate().is_ok());
    let server_handler = received.request_peer();
    let server_sibling = server_handler.clone();
    let client_handler = sent.request_peer();
    assert!(server_handler.certificate().is_ok());
    assert!(client_handler.certificate().is_ok());
    tokio::time::sleep(Duration::from_secs(4)).await;
    // Same open connections: server sees peer expiry, client sees its own expiry.
    assert!(server_handler.certificate().is_err());
    assert!(server_sibling.certificate().is_err());
    assert!(client_handler.certificate().is_err());
    assert!(received.peer_certificate().is_err());
    assert!(sent.peer_certificate().is_err());
    assert!(received.peer_certificate().is_err());
    assert!(sent.peer_certificate().is_err());
}
#[tokio::test]
async fn handshake_limit_drops_unresponsive_connection() {
    let ca = ca();
    let s = leaf(&ca, true, -60, 3600);
    let server = server(&s, &ca);
    let (_silent, socket) = sockets().await;
    assert!(
        tokio::time::timeout(
            Duration::from_secs(1),
            server.accept(socket, Duration::from_millis(30))
        )
        .await
        .unwrap()
        .is_err()
    );
    let (_silent, socket) = sockets().await;
    assert!(server.accept(socket, Duration::ZERO).await.is_err());
}

#[tokio::test]
async fn handler_gates_bind_to_their_live_connection_and_share_closure() {
    let ca = ca();
    let s = leaf(&ca, true, -60, 3600);
    let c = leaf(&ca, false, -60, 3600);
    let server = server(&s, &ca);
    let client = client(&c, &ca);
    let (mut first, _first_client) = pair(&server, &client).await;
    // Same certificate, different live connection: closure must not leak between them.
    let (mut second, _second_client) = pair(&server, &client).await;
    let first_gate = first.request_peer();
    let sibling = first_gate.clone();
    let second_gate = second.request_peer();
    assert_eq!(
        first_gate.certificate().unwrap().fingerprint(),
        first.peer_certificate().unwrap().fingerprint()
    );
    assert_eq!(
        second_gate.certificate().unwrap().fingerprint(),
        c.fingerprint()
    );
    // Owned gates work in independently scheduled handlers without borrowing the stream.
    let tasks: Vec<_> = (0..8)
        .map(|_| {
            let gate = first_gate.clone();
            std::thread::spawn(move || {
                for _ in 0..16 {
                    assert!(gate.certificate().is_ok());
                    std::thread::yield_now();
                }
                gate
            })
        })
        .collect();
    let mut completed = Vec::new();
    for task in tasks {
        completed.push(task.join().unwrap());
    }
    drop(first);
    assert!(first_gate.certificate().is_err());
    assert!(sibling.certificate().is_err());
    for gate in completed {
        assert!(gate.certificate().is_err());
    }
    assert!(second_gate.certificate().is_ok());
    assert!(second.peer_certificate().is_ok());
    drop(second);
    assert!(second_gate.certificate().is_err());
}

#[tokio::test]
async fn handler_gates_deny_after_shutdown_eof_and_transport_error() {
    let ca = ca();
    let s = leaf(&ca, true, -60, 3600);
    let c = leaf(&ca, false, -60, 3600);
    let server = server(&s, &ca);
    let client = client(&c, &ca);
    let (mut received, mut sent) = pair(&server, &client).await;
    let received_gate = received.request_peer();
    let sent_gate = sent.request_peer();
    sent.shutdown().await.unwrap();
    assert!(sent_gate.certificate().is_err());
    assert!(sent.peer_certificate().is_err());
    let mut buffer = [0; 1];
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), received.read(&mut buffer))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    assert!(received_gate.certificate().is_err());
    assert!(received.peer_certificate().is_err());

    let (mut received, sent) = pair(&server, &client).await;
    let gate = received.request_peer();
    let sibling = gate.clone();
    // Abrupt TCP close without TLS close_notify must be observed as a TLS read error.
    drop(sent);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), received.read(&mut buffer))
            .await
            .unwrap()
            .is_err()
    );
    assert!(gate.certificate().is_err());
    assert!(sibling.certificate().is_err());
    assert!(received.peer_certificate().is_err());
}
