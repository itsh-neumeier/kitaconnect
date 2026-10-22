use kitaconnect_identity_access::{
    service_auth::{RegisteredNames, authenticate},
    service_tokens::{AuthorityConfiguration, ServiceName as Name},
};
use kitaconnect_service_transport::{
    Client, ConnectionPeer, RegisteredDnsName, Server, VerifiedConnection,
};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
};
use rustls::pki_types::{PrivatePkcs8KeyDer, ServerName};
use sqlx::{
    Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::net::{TcpListener, TcpStream};
mod support;

fn dns(name: &str) -> RegisteredDnsName {
    RegisteredDnsName::new(name).unwrap()
}
fn names() -> RegisteredNames {
    RegisteredNames::new(vec![
        (Name::IdentityAccess, dns("identity.test")),
        (Name::OrganizationConfig, dns("organization.test")),
        (Name::Gateway, dns("gateway.test")),
    ])
    .unwrap()
}

#[test]
fn names_are_explicit_exact_and_unambiguous() {
    for invalid in [
        "",
        "*.test",
        ".test",
        "identity..test",
        "identity.test.",
        "-identity.test",
        "identity-.test",
        "127.0.0.1",
        "::1",
        "ünicode.test",
        "identity_test",
        "https://identity.test",
    ] {
        assert!(RegisteredDnsName::new(invalid).is_err());
    }
    assert!(RegisteredDnsName::new(&format!("{}.test", "x".repeat(64))).is_err());
    assert_eq!(dns("IDENTITY.TEST").as_str(), "identity.test");
    assert!(RegisteredNames::new(vec![]).is_err());
    assert!(
        RegisteredNames::new(vec![
            (Name::IdentityAccess, dns("identity.test")),
            (Name::IdentityAccess, dns("another.test"))
        ])
        .is_err()
    );
    assert!(
        RegisteredNames::new(vec![
            (Name::IdentityAccess, dns("identity.test")),
            (Name::Gateway, dns("IDENTITY.TEST"))
        ])
        .is_err()
    );
}

fn ca() -> CertifiedIssuer<'static, KeyPair> {
    let mut p = CertificateParams::new(Vec::<String>::new()).unwrap();
    p.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    p.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    CertifiedIssuer::self_signed(p, KeyPair::generate().unwrap()).unwrap()
}
async fn pair(
    ca: &CertifiedIssuer<'_, KeyPair>,
    client_name: &str,
) -> (VerifiedConnection, VerifiedConnection) {
    // All keys and certificates stay in RAM; no PEM/DER files or logging.
    let make = |server: bool, name: &str| {
        let mut p = CertificateParams::new(vec![name.to_owned()]).unwrap();
        if !server {
            // A matching Common Name must never rescue a different SAN.
            p.distinguished_name
                .push(rcgen::DnType::CommonName, "identity.test");
        }
        p.not_before = OffsetDateTime::now_utc() - time::Duration::seconds(60);
        p.not_after = OffsetDateTime::now_utc() + time::Duration::hours(1);
        p.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        p.extended_key_usages = vec![if server {
            ExtendedKeyUsagePurpose::ServerAuth
        } else {
            ExtendedKeyUsagePurpose::ClientAuth
        }];
        let key = KeyPair::generate().unwrap();
        let cert = p.signed_by(&key, ca).unwrap();
        (
            vec![cert.der().clone()],
            PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
        )
    };
    let (chain, key) = make(true, "organization.test");
    let server = Server::new(chain, key, vec![ca.der().clone()]).unwrap();
    let (chain, key) = make(false, client_name);
    let client = Client::new(chain, key, vec![ca.der().clone()]).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (c, s) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept()
    );
    let (s, c) = tokio::join!(
        server.accept(s.unwrap().0, Duration::from_secs(2)),
        client.connect(
            c.unwrap(),
            ServerName::try_from("organization.test").unwrap(),
            Duration::from_secs(2)
        )
    );
    (s.unwrap(), c.unwrap())
}
async fn connect(user: &str) -> PgConnection {
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(std::env::var("KC_ORG_TEST_SOCKET").unwrap())
            .database("kc_service_auth_test")
            .username(user)
            .ssl_mode(PgSslMode::Disable),
    )
    .await
    .unwrap()
}
async fn register(c: &mut PgConnection, gate: &ConnectionPeer, shift: u64) {
    let cert = gate.certificate().unwrap();
    sqlx::query("INSERT INTO identity_access.service_certificates VALUES($1,'identity-access',$2::text::numeric,$3::text::numeric,false)")
        .bind(cert.fingerprint().as_slice()).bind((cert.not_before()+shift).to_string()).bind(cert.not_after().to_string()).execute(c).await.unwrap();
}

#[tokio::test]
#[ignore = "requires scripts/test-organization-db.mjs isolated PostgreSQL fixture"]
async fn actual_peer_authentication_is_current() {
    let mut c = connect("postgres").await;
    let mut witness = connect("postgres").await;
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut c)
        .await
        .unwrap();
    let config = AuthorityConfiguration::new([7; 16], "https://identity.test/").unwrap();
    let after = std::env::var("KC_ORG_TEST_PHASE").unwrap() == "after";
    if after {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM identity_access.service_certificates")
                .fetch_one(&mut c)
                .await
                .unwrap();
        assert!(count >= 4);
    } else {
        sqlx::query("INSERT INTO identity_access.service_authority VALUES(true,$1,$2)")
            .bind(config.installation().as_slice())
            .bind(config.issuer())
            .execute(&mut c)
            .await
            .unwrap();
        sqlx::raw_sql("INSERT INTO identity_access.service_clients VALUES('identity-access',1,true),('organization-config',1,true),('gateway',1,true)").execute(&mut c).await.unwrap();
    }
    let ca = ca();
    let (stream, _client) = pair(&ca, "IDENTITY.TEST").await;
    let gate = stream.request_peer();
    register(&mut c, &gate, 0).await;
    let registered = names();
    let peer = authenticate(&mut c, &config, &registered, Name::IdentityAccess, &gate)
        .await
        .unwrap();
    assert_eq!(peer.service(), Name::IdentityAccess);
    assert_eq!(
        peer.certificate(),
        gate.certificate().unwrap().fingerprint()
    );
    assert!(
        authenticate(&mut c, &config, &registered, Name::Gateway, &gate)
            .await
            .is_err()
    );
    let missing = RegisteredNames::new(vec![(Name::Gateway, dns("gateway.test"))]).unwrap();
    assert!(
        authenticate(&mut c, &config, &missing, Name::IdentityAccess, &gate)
            .await
            .is_err()
    );
    let wrong_name = RegisteredNames::new(vec![(Name::IdentityAccess, dns("wrong.test"))]).unwrap();
    assert!(
        authenticate(&mut c, &config, &wrong_name, Name::IdentityAccess, &gate)
            .await
            .is_err()
    );

    for wrong in [
        AuthorityConfiguration::new([8; 16], config.issuer()).unwrap(),
        AuthorityConfiguration::new([7; 16], "https://wrong.test/").unwrap(),
    ] {
        assert!(
            authenticate(&mut c, &wrong, &registered, Name::IdentityAccess, &gate)
                .await
                .is_err()
        );
        support::assert_idle(&mut witness, pid).await;
    }
    let (replacement, _replacement_client) = pair(&ca, "identity.test").await;
    let fresh = replacement.request_peer();
    // A trusted certificate with the right name still needs its exact active registration.
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, pid).await;
    register(&mut c, &fresh, 0).await;
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_ok()
    );
    sqlx::query(
        "UPDATE identity_access.service_certificates SET revoked=true WHERE fingerprint=$1",
    )
    .bind(gate.certificate().unwrap().fingerprint().as_slice())
    .execute(&mut c)
    .await
    .unwrap();
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &gate)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, pid).await;
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_ok()
    );

    sqlx::raw_sql("UPDATE identity_access.service_clients SET active=false,version=version+1 WHERE name='identity-access'").execute(&mut c).await.unwrap();
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_err()
    );
    support::assert_idle(&mut witness, pid).await;
    sqlx::raw_sql("UPDATE identity_access.service_clients SET active=true,version=version+1 WHERE name='identity-access'").execute(&mut c).await.unwrap();
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_ok()
    );
    let (wrong_stream, _wrong_client) = pair(&ca, "other.test").await;
    let wrong_gate = wrong_stream.request_peer();
    register(&mut c, &wrong_gate, 0).await;
    // Exact active fingerprint and matching Common Name, but wrong actual SAN.
    assert!(
        authenticate(
            &mut c,
            &config,
            &registered,
            Name::IdentityAccess,
            &wrong_gate
        )
        .await
        .is_err()
    );
    let (time_stream, _time_client) = pair(&ca, "identity.test").await;
    let time_gate = time_stream.request_peer();
    register(&mut c, &time_gate, 1).await;
    assert!(
        authenticate(
            &mut c,
            &config,
            &registered,
            Name::IdentityAccess,
            &time_gate
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, pid).await;
    let mut runtime = connect("kc_service_auth_runtime").await;
    let runtime_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut runtime)
        .await
        .unwrap();
    assert!(
        authenticate(
            &mut runtime,
            &config,
            &registered,
            Name::IdentityAccess,
            &fresh
        )
        .await
        .is_err()
    );
    support::assert_idle(&mut witness, runtime_pid).await;
    let mut tx = c.begin().await.unwrap();
    assert!(
        authenticate(&mut tx, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let mut unavailable = connect("postgres").await;
    let unavailable_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut unavailable)
        .await
        .unwrap();
    let terminated: bool = sqlx::query_scalar("SELECT pg_terminate_backend($1)")
        .bind(unavailable_pid)
        .fetch_one(&mut witness)
        .await
        .unwrap();
    assert!(terminated);
    assert!(
        authenticate(
            &mut unavailable,
            &config,
            &registered,
            Name::IdentityAccess,
            &fresh
        )
        .await
        .is_err()
    );
    drop(replacement);
    assert!(
        authenticate(&mut c, &config, &registered, Name::IdentityAccess, &fresh)
            .await
            .is_err()
    );
    drop(stream);
}
