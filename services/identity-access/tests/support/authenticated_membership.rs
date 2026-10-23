//! Synthetic cross-service fixture. Identity and organization use separate databases.
use super::*;
use kitaconnect_organization_config::{
    FacilityId,
    authenticated_membership::{Configuration, MembershipReader, ReadError, ReadLimits},
    membership::{CurrentMembership, IdentityReference},
    postgres,
    session_policy::{self, SessionPolicyLimits},
};
use kitaconnect_service_transport::ConnectionPeer;
use std::net::SocketAddr;

struct Fixture<'a, 'issuer> {
    http: &'a Arc<OAuthServer>,
    organization: &'a Leaf,
    ca: &'a CertifiedIssuer<'issuer, KeyPair>,
}
fn org_options(user: &str) -> PgConnectOptions {
    options(user)
        .database("kc_authenticated_membership_test")
        .application_name("kc_membership_fixture")
}
fn read_limits() -> ReadLimits {
    ReadLimits {
        parallel: 1,
        response_bytes: 2048,
        connect_time: Duration::from_secs(1),
        database_time: Duration::from_millis(500),
        total_time: Duration::from_secs(3),
    }
}
fn person() -> IdentityReference {
    IdentityReference::new(1).unwrap()
}
fn facility(after: bool) -> FacilityId {
    FacilityId::new(if after { 101 } else { 1 }).unwrap()
}
fn serve_once(
    http: Arc<OAuthServer>,
    listener: Arc<TcpListener>,
) -> JoinHandle<Result<(), kitaconnect_identity_access::service_http::HttpError>> {
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        http.serve(socket).await
    })
}
async fn finish(
    task: JoinHandle<Result<(), kitaconnect_identity_access::service_http::HttpError>>,
) {
    let mut task = task;
    // Await ownership even if a test regression fails to connect; never leave fixture tasks running.
    match timeout(Duration::from_secs(2), &mut task).await {
        Ok(joined) => {
            let _ = joined.expect("fixture task must not panic");
        }
        Err(_) => {
            task.abort();
            let _ = task.await;
            panic!("fixture server did not finish");
        }
    }
}
impl Fixture<'_, '_> {
    fn reader(
        &self,
        address: SocketAddr,
        pin: [u8; 32],
        database: PgConnectOptions,
        limits: ReadLimits,
    ) -> MembershipReader {
        let client = OAuthHttpClient::new(
            Client::new(
                self.organization.chain.clone(),
                self.organization.key.clone_key(),
                vec![self.ca.der().clone()],
            )
            .unwrap(),
            RegisteredDnsName::new("identity.test").unwrap(),
            ClientLimits {
                parallel: 1,
                form_bytes: 512,
                response_bytes: 2048,
                headers: 16,
                header_bytes: 8192,
                handshake_time: Duration::from_secs(1),
                header_time: Duration::from_secs(1),
                body_time: Duration::from_secs(1),
                total_time: Duration::from_secs(2),
            },
        )
        .unwrap();
        MembershipReader::new(
            client,
            Configuration {
                address,
                issuer: config().issuer().to_owned(),
                caller_name: RegisteredDnsName::new("identity.test").unwrap(),
                server_certificate: pin,
                limits,
            },
            database,
        )
        .unwrap()
    }
    async fn incoming(&self, caller: &Leaf) -> (VerifiedConnection, VerifiedConnection) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let client_socket = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (socket, _) = listener.accept().await.unwrap();
        let server = Server::new(
            self.organization.chain.clone(),
            self.organization.key.clone_key(),
            vec![self.ca.der().clone()],
        )
        .unwrap();
        let client = Client::new(
            caller.chain.clone(),
            caller.key.clone_key(),
            vec![self.ca.der().clone()],
        )
        .unwrap();
        let (client, server) = tokio::join!(
            client.connect(
                client_socket,
                ServerName::try_from("organization.test").unwrap(),
                Duration::from_secs(1)
            ),
            server.accept(socket, Duration::from_secs(1))
        );
        (client.unwrap(), server.unwrap())
    }
    async fn read(
        &self,
        peer: &ConnectionPeer,
        token: &[u8; 32],
        facility: FacilityId,
        database: PgConnectOptions,
        pin: [u8; 32],
    ) -> Result<Option<CurrentMembership>, ReadError> {
        let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
        let reader = self.reader(listener.local_addr().unwrap(), pin, database, read_limits());
        let task = serve_once(self.http.clone(), listener);
        let result = reader.read(peer, token, person(), facility).await;
        finish(task).await;
        result
    }
}

pub(super) async fn check(
    http: &Arc<OAuthServer>,
    identity: &Leaf,
    organization: &Leaf,
    gateway: &Leaf,
    ca: &CertifiedIssuer<'_, KeyPair>,
    after: bool,
) {
    let f = Fixture {
        http,
        organization,
        ca,
    };
    let mut admin = PgConnection::connect_with(&org_options("postgres"))
        .await
        .unwrap();
    assert!(
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT to_regclass('identity_access.service_tokens')::text"
        )
        .fetch_one(&mut admin)
        .await
        .unwrap()
        .is_none()
    );
    if after {
        let version: String=sqlx::query_scalar("SELECT trim_scale(version)::text FROM organization_config.memberships WHERE identity_ref=1 AND facility_id=1 AND NOT active").fetch_one(&mut admin).await.unwrap();
        assert_eq!(version, "2");
    }
    let site = facility(after);
    let other = FacilityId::new(site.value() + 1).unwrap();
    for id in [site, other] {
        postgres::register_facility(&mut admin, id, None)
            .await
            .unwrap();
        session_policy::write(&mut admin, id, None, SessionPolicyLimits::default())
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO organization_config.memberships(identity_ref,facility_id,version,active) VALUES(1,$1::text::numeric,1,true)").bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    let response = library_request(
        http.clone(),
        identity,
        ca,
        OAuthEndpoint::Token,
        b"client_id=identity-access&grant_type=client_credentials&scope=membership%3Aread",
    )
    .await;
    let issued = TokenResponse::parse(
        response.status(),
        response.body(),
        "membership:read",
        ResponseLimits::new(2048).unwrap(),
    )
    .unwrap();
    use base64::Engine as _;
    let mut token = zeroize::Zeroizing::new([0; 32]);
    assert_eq!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode_slice(issued.expose_token(), token.as_mut())
            .unwrap(),
        32
    );
    let (_caller, incoming) = f.incoming(identity).await;
    let peer = incoming.request_peer();
    let current = f
        .read(
            &peer,
            &token,
            site,
            org_options("postgres"),
            identity.digest,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (current.identity(), current.facility(), current.version()),
        (person(), site, 1)
    );
    assert_eq!(current.policy().limits(), SessionPolicyLimits::default());
    assert!(
        f.read(
            &peer,
            &token,
            other,
            org_options("postgres"),
            identity.digest
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        f.read(
            &peer,
            &token,
            site,
            org_options("kc_authenticated_membership_runtime"),
            identity.digest
        )
        .await
        .unwrap()
        .is_none()
    );
    assert_eq!(
        f.read(&peer, &token, site, org_options("postgres"), [0; 32])
            .await,
        Err(ReadError::Rejected)
    );
    assert_eq!(
        f.read(
            &peer,
            &token,
            site,
            org_options("postgres").database("kc_missing_membership_database"),
            identity.digest
        )
        .await,
        Err(ReadError::Unavailable)
    );
    assert_eq!(
        f.read(
            &peer,
            &[9; 32],
            site,
            org_options("postgres"),
            identity.digest
        )
        .await,
        Err(ReadError::Rejected)
    );

    // A second genuine TLS certificate with the same SAN cannot use the first certificate's token.
    let changed = leaf(ca, "identity.test");
    let (_changed_caller, changed_incoming) = f.incoming(&changed).await;
    assert!(
        f.read(
            &changed_incoming.request_peer(),
            &token,
            site,
            org_options("postgres"),
            identity.digest
        )
        .await
        .is_err()
    );
    let (_foreign_caller, foreign_incoming) = f.incoming(gateway).await;
    let unused = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reader = f.reader(
        unused.local_addr().unwrap(),
        identity.digest,
        org_options("postgres"),
        read_limits(),
    );
    assert_eq!(
        reader
            .read(&foreign_incoming.request_peer(), &token, person(), site)
            .await,
        Err(ReadError::Rejected)
    );
    let closed = changed_incoming.request_peer();
    drop(changed_incoming);
    assert_eq!(
        reader.read(&closed, &token, person(), site).await,
        Err(ReadError::Rejected)
    );

    // A server that completes TLS but never answers HTTP cannot exhaust the whole-operation budget.
    let stalled_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let timed = f.reader(
        stalled_listener.local_addr().unwrap(),
        identity.digest,
        org_options("postgres"),
        ReadLimits {
            connect_time: Duration::from_millis(100),
            database_time: Duration::from_millis(100),
            total_time: Duration::from_millis(250),
            ..read_limits()
        },
    );
    let tls = Server::new(
        identity.chain.clone(),
        identity.key.clone_key(),
        vec![ca.der().clone()],
    )
    .unwrap();
    let stalled = tokio::spawn(async move {
        let (socket, _) = stalled_listener.accept().await.unwrap();
        let mut connection = tls.accept(socket, Duration::from_secs(1)).await.unwrap();
        let gate = connection.request_peer();
        let mut buffer = zeroize::Zeroizing::new([0; 512]);
        while matches!(connection.read(buffer.as_mut()).await,Ok(n) if n>0) {}
        assert!(gate.certificate().is_err());
    });
    assert_eq!(
        timed.read(&peer, &token, person(), site).await,
        Err(ReadError::Unavailable)
    );
    timeout(Duration::from_secs(1), stalled)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        timed.read(&closed, &token, person(), site).await,
        Err(ReadError::Rejected)
    );

    // Independent witness: both timeout and caller cancellation remove the waiting backend
    // while its blocker is still held, release the budget, and allow the same reader to recover.
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let reader = Arc::new(f.reader(
        listener.local_addr().unwrap(),
        identity.digest,
        org_options("postgres"),
        read_limits(),
    ));
    for cancel in [false, true] {
        let mut blocker = PgConnection::connect_with(&org_options("postgres"))
            .await
            .unwrap();
        blocker
            .execute("BEGIN; LOCK TABLE organization_config.memberships IN ACCESS EXCLUSIVE MODE")
            .await
            .unwrap();
        let server = serve_once(http.clone(), listener.clone());
        let r = reader.clone();
        let p = peer.clone();
        let t = *token;
        let task = tokio::spawn(async move { r.read(&p, &t, person(), site).await });
        let pid: i32=timeout(Duration::from_secs(2),async {
            loop {
                let pid: Option<i32>=sqlx::query_scalar("SELECT pid FROM pg_stat_activity WHERE datname='kc_authenticated_membership_test' AND application_name='kc_authenticated_membership_request' AND wait_event_type='Lock'").fetch_optional(&mut admin).await.unwrap();
                if let Some(pid)=pid { break pid; } sleep(Duration::from_millis(5)).await;
            }
        }).await.unwrap();
        assert_eq!(
            reader.read(&peer, &token, person(), site).await,
            Err(ReadError::Unavailable)
        );
        if cancel {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            assert_eq!(task.await.unwrap(), Err(ReadError::Unavailable));
        }
        finish(server).await;
        timeout(Duration::from_secs(2), async {
            while sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT FROM pg_stat_activity WHERE pid=$1)",
            )
            .bind(pid)
            .fetch_one(&mut admin)
            .await
            .unwrap()
            {
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("backend must disappear while blocker remains held");
        blocker.execute("ROLLBACK").await.unwrap();
        let server = serve_once(http.clone(), listener.clone());
        assert!(
            reader
                .read(&peer, &token, person(), site)
                .await
                .unwrap()
                .is_some()
        );
        finish(server).await;
    }
    sqlx::query("UPDATE organization_config.memberships SET active=false,version=version+1 WHERE identity_ref=1 AND facility_id=$1::text::numeric").bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    assert!(
        f.read(
            &peer,
            &token,
            site,
            org_options("postgres"),
            identity.digest
        )
        .await
        .unwrap()
        .is_none()
    );
    let mut identity_admin = PgConnection::connect_with(&options("postgres"))
        .await
        .unwrap();
    sqlx::query("UPDATE identity_access.service_grants SET active=false,version=version+1 WHERE client='identity-access' AND target='organization-config' AND scope='membership:read'").execute(&mut identity_admin).await.unwrap();
    assert_eq!(
        f.read(
            &peer,
            &token,
            site,
            org_options("postgres"),
            identity.digest
        )
        .await,
        Err(ReadError::Rejected)
    );
    sqlx::query("UPDATE identity_access.service_grants SET active=true,version=version+1 WHERE client='identity-access' AND target='organization-config' AND scope='membership:read'").execute(&mut identity_admin).await.unwrap();
}
