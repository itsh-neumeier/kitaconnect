//! Actual organization HTTP -> identity HTTP -> separate organization DB, synthetic only.
use super::*;
use base64::Engine as _;
use kitaconnect_identity_access::{
    IdentityId,
    login::LoginFacilityId,
    organization_http::{
        MembershipReader as Consumer, ReadError as ConsumerError, ReadLimits as ConsumerLimits,
    },
};
use kitaconnect_organization_config::service_http::{
    HttpError as OrgHttpError, Limits as OrgLimits, MembershipServer,
};
use kitaconnect_service_transport::membership_http::{
    ClientLimits as MemberLimits, MembershipHttpClient,
};

const PATH: &str = "/internal/v1/membership/read";
const JSON: &str = "Content-Type: application/json\r\n";
struct OrgWire {
    stream: BufReader<VerifiedConnection>,
    task: JoinHandle<Result<(), OrgHttpError>>,
}
impl Drop for OrgWire {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl OrgWire {
    async fn request(&mut self, method: &str, path: &str, headers: &str, body: &str) -> Reply {
        let mut request = zeroize::Zeroizing::new(format!(
            "{method} {path} HTTP/1.1\r\nHost: organization.test\r\nContent-Length: {}\r\n{headers}\r\n",
            body.len()
        ));
        request.push_str(body);
        self.stream
            .get_mut()
            .write_all(request.as_bytes())
            .await
            .unwrap();
        self.stream.get_mut().flush().await.unwrap();
        self.receive().await
    }
    async fn receive(&mut self) -> Reply {
        timeout(Duration::from_secs(5), async {
            let mut line = String::new();
            self.stream.read_line(&mut line).await.unwrap();
            let status = line
                .split_whitespace()
                .nth(1)
                .expect("actual HTTP status")
                .parse()
                .unwrap();
            let mut headers = String::new();
            let mut length = 0;
            loop {
                line.clear();
                assert!(self.stream.read_line(&mut line).await.unwrap() > 0);
                if line == "\r\n" {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().unwrap();
                }
                assert!(!lower.starts_with("transfer-encoding:"));
                headers.push_str(&lower);
            }
            assert!(length <= 2048);
            let mut body = vec![0; length];
            self.stream.read_exact(&mut body).await.unwrap();
            assert!(headers.contains("cache-control: no-store\r\n"));
            assert!(headers.contains("pragma: no-cache\r\n"));
            Reply {
                status,
                headers,
                body,
            }
        })
        .await
        .expect("bounded actual membership response")
    }
}
fn org_server(
    f: &Fixture<'_, '_>,
    address: SocketAddr,
    pin: [u8; 32],
    database: PgConnectOptions,
) -> Arc<MembershipServer> {
    Arc::new(
        MembershipServer::new(
            Server::new(
                f.organization.chain.clone(),
                f.organization.key.clone_key(),
                vec![f.ca.der().clone()],
            )
            .unwrap(),
            f.reader(address, pin, database, read_limits()),
            OrgLimits {
                connections: 2,
                requests: 1,
                body_bytes: 128,
                headers: 16,
                header_bytes: 8192,
                handshake_time: Duration::from_secs(1),
                header_time: Duration::from_secs(10),
                request_time: Duration::from_secs(2),
                connection_time: Duration::from_secs(60),
            },
        )
        .unwrap(),
    )
}
async fn org_wire(server: Arc<MembershipServer>, caller: &Leaf, f: &Fixture<'_, '_>) -> OrgWire {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (accepted, _) = listener.accept().await.unwrap();
    let task = tokio::spawn(server.serve(accepted));
    let stream = Client::new(
        caller.chain.clone(),
        caller.key.clone_key(),
        vec![f.ca.der().clone()],
    )
    .unwrap()
    .connect(
        socket,
        ServerName::try_from("organization.test").unwrap(),
        Duration::from_secs(1),
    )
    .await
    .unwrap();
    OrgWire {
        stream: BufReader::new(stream),
        task,
    }
}
fn body(site: FacilityId) -> String {
    format!(r#"{{"identity_ref":"1","facility_id":"{}"}}"#, site.value())
}
#[track_caller]
fn problem(reply: &Reply, status: u16) {
    assert_eq!(reply.status, status);
    assert!(
        reply
            .headers
            .contains("content-type: application/problem+json\r\n")
    );
    assert!(reply.headers.contains("connection: close\r\n"));
    let value = reply.json();
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert_eq!(value["type"], "about:blank");
    assert_eq!(value["status"], status);
}
pub(super) struct Running {
    wire: OrgWire,
    headers: zeroize::Zeroizing<String>,
    authority: JoinHandle<()>,
    organization: JoinHandle<()>,
    reader: Consumer,
    token: zeroize::Zeroizing<[u8; 32]>,
}
impl Drop for Running {
    fn drop(&mut self) {
        self.authority.abort();
        self.organization.abort();
    }
}
impl Running {
    pub(super) async fn absent(&mut self, site: FacilityId) {
        let reply = self
            .wire
            .request("POST", PATH, &self.headers, &body(site))
            .await;
        assert_eq!(reply.status, 200);
        assert_eq!(reply.json(), json!({"membership":null}));
        assert!(
            self.reader
                .read(
                    &self.token,
                    IdentityId::new(1).unwrap(),
                    LoginFacilityId::new(site.value()).unwrap()
                )
                .await
                .unwrap()
                .is_none()
        );
    }
    pub(super) async fn rejected(&mut self, site: FacilityId) {
        let reply = self
            .wire
            .request("POST", PATH, &self.headers, &body(site))
            .await;
        problem(&reply, 401);
        assert!(
            reply.headers.contains(
                "www-authenticate: bearer realm=\"service\", error=\"invalid_token\"\r\n"
            )
        );
        assert_eq!(
            self.reader
                .read(
                    &self.token,
                    IdentityId::new(1).unwrap(),
                    LoginFacilityId::new(site.value()).unwrap()
                )
                .await,
            Err(ConsumerError::Rejected)
        );
    }
    pub(super) async fn finish(mut self) {
        let _ = timeout(Duration::from_secs(1), &mut self.wire.task)
            .await
            .expect("router finishes after 401")
            .unwrap();
        self.authority.abort();
        let _ = (&mut self.authority).await;
        self.organization.abort();
        let _ = (&mut self.organization).await;
    }
}
pub(super) async fn check(
    f: &Fixture<'_, '_>,
    identity: &Leaf,
    gateway: &Leaf,
    token: &[u8; 32],
    site: FacilityId,
    other: FacilityId,
) -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let authority = f.http.clone();
    let task = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let _ = authority.clone().serve(socket).await;
        }
    });
    let server = org_server(f, address, identity.digest, org_options("postgres"));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reader = consumer(
        f,
        identity,
        listener.local_addr().unwrap(),
        f.organization.digest,
        consumer_limits(),
    );
    let serving_organization = server.clone();
    let organization = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let _ = serving_organization.clone().serve(socket).await;
        }
    });
    let headers = zeroize::Zeroizing::new(format!(
        "{JSON}Authorization: Bearer {}\r\n",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token)
    ));
    let mut running = Running {
        wire: org_wire(server.clone(), identity, f).await,
        headers,
        authority: task,
        organization,
        reader,
        token: zeroize::Zeroizing::new(*token),
    };
    let reply = running
        .wire
        .request("POST", PATH, &running.headers, &body(site))
        .await;
    assert_eq!(reply.status, 200);
    assert!(reply.headers.contains("content-type: application/json\r\n"));
    assert_eq!(
        reply.json(),
        json!({"membership": {
            "identity_ref":"1", "facility_id":site.value().to_string(), "membership_version":"1", "policy_version":"1",
            "idle_seconds":900, "absolute_seconds":28800, "step_up_seconds":300
        }})
    );
    let snapshot = running
        .reader
        .read(
            token,
            IdentityId::new(1).unwrap(),
            LoginFacilityId::new(site.value()).unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (snapshot.identity().value(), snapshot.facility().value()),
        (1, site.value())
    );
    assert_eq!(
        (snapshot.membership_version(), snapshot.policy_version()),
        (1, 1)
    );
    assert_eq!(
        (
            snapshot.idle_seconds(),
            snapshot.absolute_seconds(),
            snapshot.step_up_seconds()
        ),
        (900, 28800, 300)
    );
    running.absent(other).await;

    // Every router error carries only Problem Details and closes the actual connection.
    for (method, path, headers, payload, status) in [
        ("POST", PATH, JSON.to_owned(), body(site), 401),
        ("GET", PATH, running.headers.to_string(), String::new(), 405),
        (
            "POST",
            "/unknown",
            running.headers.to_string(),
            body(site),
            404,
        ),
        (
            "POST",
            PATH,
            running
                .headers
                .replace(JSON, "Content-Type: text/plain\r\n"),
            body(site),
            415,
        ),
        (
            "POST",
            PATH,
            running.headers.to_string(),
            r#"{"identity_ref":"1","facility_id":"01"}"#.to_owned(),
            400,
        ),
        (
            "POST",
            PATH,
            running.headers.to_string(),
            r#"{"identity_ref":"1","identity\u005fref":"1","facility_id":"1"}"#.to_owned(),
            400,
        ),
        (
            "POST",
            PATH,
            format!("{}Cookie: token=synthetic\r\n", running.headers.as_str()),
            body(site),
            400,
        ),
        (
            "POST",
            PATH,
            format!(
                "{}X-Forwarded-Client-Cert: synthetic\r\n",
                running.headers.as_str()
            ),
            body(site),
            400,
        ),
        (
            "POST",
            "/internal/v1/membership/read?token=synthetic",
            running.headers.to_string(),
            body(site),
            400,
        ),
        (
            "POST",
            PATH,
            format!(
                "{}Authorization: Bearer synthetic\r\n",
                running.headers.as_str()
            ),
            body(site),
            400,
        ),
        (
            "POST",
            PATH,
            running.headers.to_string(),
            " ".repeat(129),
            413,
        ),
    ] {
        let mut wire = org_wire(server.clone(), identity, f).await;
        let reply = wire.request(method, path, &headers, &payload).await;
        problem(&reply, status);
        if status == 405 {
            assert!(reply.headers.contains("allow: post\r\n"));
        }
        if status == 401 {
            assert!(
                reply
                    .headers
                    .contains("www-authenticate: bearer realm=\"service\"\r\n")
            );
        }
        timeout(Duration::from_secs(1), &mut wire.task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
    // Foreign SAN is rejected locally. A same-SAN replacement reaches introspection,
    // whose different token/certificate binding is an unavailable protocol check.
    for (caller, expected) in [(gateway, 401), (&leaf(f.ca, "identity.test"), 503)] {
        let mut wire = org_wire(server.clone(), caller, f).await;
        problem(
            &wire
                .request("POST", PATH, &running.headers, &body(site))
                .await,
            expected,
        );
        let _ = timeout(Duration::from_secs(1), &mut wire.task)
            .await
            .unwrap()
            .unwrap();
    }
    for (database, pin, expected) in [
        (
            org_options("kc_authenticated_membership_runtime"),
            identity.digest,
            200,
        ),
        (
            org_options("postgres").database("kc_nonexistent_router_test"),
            identity.digest,
            503,
        ),
        (org_options("postgres"), [0; 32], 401),
    ] {
        let mut wire = org_wire(org_server(f, address, pin, database), identity, f).await;
        let reply = wire
            .request("POST", PATH, &running.headers, &body(site))
            .await;
        if expected == 200 {
            assert_eq!(reply.status, 200);
            assert_eq!(reply.json(), json!({"membership":null}));
        } else {
            problem(&reply, expected);
        }
    }

    // One incomplete body occupies the shared request budget. Another real connection sees 429.
    let mut slow = org_wire(server.clone(), identity, f).await;
    let partial = zeroize::Zeroizing::new(format!(
        "POST {PATH} HTTP/1.1\r\nHost: organization.test\r\nContent-Length: 100\r\nExpect: 100-continue\r\n{}\r\n",
        running.headers.as_str()
    ));
    slow.stream
        .get_mut()
        .write_all(partial.as_bytes())
        .await
        .unwrap();
    slow.stream.get_mut().flush().await.unwrap();
    // Continue is emitted only once the permitted handler actually polls the body.
    timeout(Duration::from_secs(1), async {
        let mut line = String::new();
        slow.stream.read_line(&mut line).await.unwrap();
        assert_eq!(line, "HTTP/1.1 100 Continue\r\n");
        line.clear();
        slow.stream.read_line(&mut line).await.unwrap();
        assert_eq!(line, "\r\n");
    })
    .await
    .expect("independent witness that the slow handler owns its permit");
    // The third connection cannot start TLS while both connection permits remain held.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let _third = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (accepted, _) = listener.accept().await.unwrap();
    assert!(server.clone().serve(accepted).await.is_err());
    let reply = running
        .wire
        .request("POST", PATH, &running.headers, &body(site))
        .await;
    problem(&reply, 429);
    let reply = slow.receive().await;
    problem(&reply, 503);
    let _ = timeout(Duration::from_secs(1), &mut slow.task)
        .await
        .unwrap()
        .unwrap();
    let _ = timeout(Duration::from_secs(1), &mut running.wire.task)
        .await
        .unwrap()
        .unwrap();
    running.wire = org_wire(server.clone(), identity, f).await;
    assert_eq!(
        running
            .wire
            .request("POST", PATH, &running.headers, &body(site))
            .await
            .status,
        200
    );
    outer_budget_and_cancellation(f, identity, &server, token, site).await;
    running
}

fn consumer_limits() -> ConsumerLimits {
    ConsumerLimits {
        parallel: 1,
        connect_time: Duration::from_secs(1),
        total_time: Duration::from_secs(3),
    }
}
fn consumer(
    f: &Fixture<'_, '_>,
    identity: &Leaf,
    address: SocketAddr,
    pin: [u8; 32],
    limits: ConsumerLimits,
) -> Consumer {
    let client = MembershipHttpClient::new(
        Client::new(
            identity.chain.clone(),
            identity.key.clone_key(),
            vec![f.ca.der().clone()],
        )
        .unwrap(),
        RegisteredDnsName::new("organization.test").unwrap(),
        pin,
        MemberLimits {
            parallel: 1,
            request_bytes: 128,
            response_bytes: 2048,
            headers: 16,
            header_bytes: 8192,
            handshake_time: Duration::from_secs(1),
            header_time: Duration::from_secs(2),
            body_time: Duration::from_secs(2),
            total_time: Duration::from_secs(3),
        },
    )
    .unwrap();
    Consumer::new(client, address, limits).unwrap()
}
async fn outer_budget_and_cancellation(
    f: &Fixture<'_, '_>,
    identity: &Leaf,
    server: &Arc<MembershipServer>,
    token: &[u8; 32],
    site: FacilityId,
) {
    for cancel in [false, true] {
        let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
        let reader = Arc::new(consumer(
            f,
            identity,
            listener.local_addr().unwrap(),
            f.organization.digest,
            ConsumerLimits {
                parallel: 1,
                connect_time: Duration::from_millis(100),
                total_time: Duration::from_millis(500),
            },
        ));
        let (started_tx, started) = tokio::sync::oneshot::channel();
        let l = listener.clone();
        let silent = tokio::spawn(async move {
            let (mut socket, _) = l.accept().await.unwrap();
            started_tx.send(()).unwrap();
            let mut bytes = [0; 2048];
            while matches!(socket.read(&mut bytes).await,Ok(n) if n>0) {}
        });
        let r = reader.clone();
        let secret = zeroize::Zeroizing::new(*token);
        let facility = LoginFacilityId::new(site.value()).unwrap();
        let caller =
            tokio::spawn(
                async move { r.read(&secret, IdentityId::new(1).unwrap(), facility).await },
            );
        timeout(Duration::from_secs(1), started)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            reader
                .read(token, IdentityId::new(1).unwrap(), facility)
                .await,
            Err(ConsumerError::Unavailable)
        );
        if cancel {
            caller.abort();
            assert!(caller.await.unwrap_err().is_cancelled());
        } else {
            assert_eq!(caller.await.unwrap(), Err(ConsumerError::Unavailable));
        }
        timeout(Duration::from_secs(1), silent)
            .await
            .expect("outer abort closes the actual TCP socket")
            .unwrap();
        let http = server.clone();
        let l = listener.clone();
        let service = tokio::spawn(async move {
            let (socket, _) = l.accept().await.unwrap();
            http.serve(socket).await
        });
        assert!(
            reader
                .read(token, IdentityId::new(1).unwrap(), facility)
                .await
                .unwrap()
                .is_some()
        );
        let _ = timeout(Duration::from_secs(1), service)
            .await
            .unwrap()
            .unwrap();
    }
}
