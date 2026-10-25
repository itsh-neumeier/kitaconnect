//! Fixed membership client against actual synthetic TLS/HTTP peers; no authorization proof.
use super::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kitaconnect_service_transport::membership_http::{
    ClientLimits as Limits, HttpError, MembershipHttpClient, MembershipReply,
};
fn limits() -> Limits {
    Limits {
        parallel: 1,
        request_bytes: 128,
        response_bytes: 256,
        headers: 16,
        header_bytes: 8192,
        handshake_time: Duration::from_secs(1),
        header_time: Duration::from_secs(1),
        body_time: Duration::from_secs(1),
        total_time: Duration::from_secs(2),
    }
}
fn client(f: &Fixture, limits: Limits, pin: [u8; 32]) -> MembershipHttpClient {
    MembershipHttpClient::new(
        Client::new(f.client.chain(), f.client.key(), vec![f.ca.der().clone()]).unwrap(),
        RegisteredDnsName::new("identity.test").unwrap(),
        pin,
        limits,
    )
    .unwrap()
}
fn pin(f: &Fixture) -> [u8; 32] {
    Sha256::digest(f.server.cert.der().as_ref()).into()
}
async fn request(stream: VerifiedConnection) -> BufReader<VerifiedConnection> {
    let mut r = BufReader::new(stream);
    let mut line = zeroize::Zeroizing::new(String::new());
    r.read_line(&mut line).await.unwrap();
    assert!(line.as_str() == "POST /internal/v1/membership/read HTTP/1.1\r\n");
    let mut length = 0;
    let mut auth = 0;
    let mut json = false;
    loop {
        line.clear();
        assert!(r.read_line(&mut line).await.unwrap() > 0);
        if line.as_str() == "\r\n" {
            break;
        }
        let lower = zeroize::Zeroizing::new(line.to_ascii_lowercase());
        if lower.starts_with("authorization:") {
            auth += 1;
            let expected = zeroize::Zeroizing::new(format!(
                "authorization: Bearer {}\r\n",
                URL_SAFE_NO_PAD.encode([7; 32])
            ));
            assert!(line.as_str() == expected.as_str());
        }
        if let Some(l) = lower.strip_prefix("content-length:") {
            length = l.trim().parse::<usize>().unwrap();
        }
        if lower.as_str() == "content-type: application/json\r\n" {
            json = true;
        }
        assert!(
            !lower.starts_with("cookie:")
                && !lower.starts_with("forwarded:")
                && !lower.starts_with("x-forwarded-")
        );
    }
    assert!(auth == 1 && json && length <= 128);
    let mut body = vec![0; length];
    r.read_exact(&mut body).await.unwrap();
    assert!(body.as_slice() == br#"{"identity_ref":"1","facility_id":"2"}"#);
    r
}
async fn respond(server: Server, socket: TcpStream, spec: ResponseSpec) {
    let stream = server.accept(socket, Duration::from_secs(2)).await.unwrap();
    let gate = stream.request_peer();
    let mut r = request(stream).await;
    sleep(spec.header_delay).await;
    let header = format!(
        "HTTP/1.1 {} fixture\r\nContent-Length: {}\r\n{}\r\n",
        spec.status,
        spec.body.len(),
        spec.headers
    );
    if r.get_mut().write_all(header.as_bytes()).await.is_ok() {
        sleep(spec.body_delay).await;
        if spec.truncate {
            let _ = r.get_mut().write_all(&spec.body[..1]).await;
            return;
        }
        let _ = r.get_mut().write_all(&spec.body).await;
    }
    if spec.headers.contains("Connection: close\r\n") {
        return;
    }
    let mut byte = [0];
    let result = timeout(Duration::from_secs(2), r.read(&mut byte))
        .await
        .expect("client drops owned connection");
    assert!(matches!(result, Ok(0) | Err(_)));
    assert!(gate.certificate().is_err());
}
async fn call(
    f: &Fixture,
    c: &MembershipHttpClient,
    spec: ResponseSpec,
) -> Result<MembershipReply, HttpError> {
    let (out, incoming) = sockets().await;
    let (reply, ()) = tokio::join!(
        c.request(out, &[7; 32], 1, 2),
        respond(f.server(), incoming, spec)
    );
    reply
}
fn response() -> ResponseSpec {
    let mut s = ResponseSpec::json();
    s.body = br#"{"membership":null}"#.to_vec();
    s
}
#[tokio::test]
async fn fixed_membership_request_and_problem_response_use_actual_tls() {
    let f = Fixture::new();
    let c = client(&f, limits(), pin(&f));
    let reply = call(&f, &c, response()).await.unwrap();
    assert!(reply.status() == 200 && reply.body() == br#"{"membership":null}"#);
    assert_eq!(format!("{reply:?}"), "MembershipReply([REDACTED])");
    let mut spec = response();
    spec.status = 401;
    spec.headers = spec
        .headers
        .replace("application/json", "application/problem+json");
    spec.body = br#"{"type":"about:blank","title":"Unauthorized","status":401}"#.to_vec();
    spec.headers.push_str("Connection: close\r\n");
    assert_eq!(call(&f, &c, spec).await.unwrap().status(), 401);
    for parallel in [0, tokio::sync::Semaphore::MAX_PERMITS + 1] {
        assert!(
            MembershipHttpClient::new(
                Client::new(f.client.chain(), f.client.key(), vec![f.ca.der().clone()]).unwrap(),
                RegisteredDnsName::new("identity.test").unwrap(),
                pin(&f),
                Limits {
                    parallel,
                    ..limits()
                }
            )
            .is_err()
        );
    }
}
#[tokio::test]
async fn changed_server_pin_and_invalid_request_send_no_application_bytes() {
    let f = Fixture::new();
    let c = client(&f, limits(), [0; 32]);
    let (out, incoming) = sockets().await;
    let server = f.server();
    let (reply, ()) = tokio::join!(c.request(out, &[7; 32], 1, 2), async {
        let mut stream = server
            .accept(incoming, Duration::from_secs(1))
            .await
            .unwrap();
        let mut byte = [0];
        let result = timeout(Duration::from_secs(1), stream.read(&mut byte))
            .await
            .unwrap();
        assert!(matches!(result, Ok(0) | Err(_)));
    });
    assert!(reply.is_err());
    for (id, l) in [
        (0, limits()),
        (
            1,
            Limits {
                request_bytes: 1,
                ..limits()
            },
        ),
    ] {
        let c = client(&f, l, pin(&f));
        let (out, mut incoming) = sockets().await;
        assert!(c.request(out, &[7; 32], id, 2).await.is_err());
        let mut byte = [0];
        assert_eq!(
            timeout(Duration::from_secs(1), incoming.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
    }
}
#[tokio::test]
async fn headers_bodies_and_phase_deadlines_are_bounded() {
    let f = Fixture::new();
    let c = client(&f, limits(), pin(&f));
    for case in 0..15 {
        let mut s = response();
        match case {
            0 => s.headers = s.headers.replace("no-store", "public"),
            1 => s.headers.push_str("Cache-Control: no-store\r\n"),
            2 => {
                s.headers = s
                    .headers
                    .replace("application/json", "application/problem+json")
            }
            3 => s.headers.push_str("Content-Type: application/json\r\n"),
            4 => s.status = 302,
            5 => s.body = vec![b'x'; 257],
            6 => s.headers.push_str(&"X-Test: bounded\r\n".repeat(17)),
            7 => s
                .headers
                .push_str(&format!("X-Large: {}\r\n", "x".repeat(9000))),
            8 => s.headers = s.headers.replace("Pragma: no-cache\r\n", ""),
            9 => s.truncate = true,
            10 => s.headers.push_str("Content-Encoding: gzip\r\n"),
            11 => s.headers.push_str("Set-Cookie: synthetic=true\r\n"),
            12 => s.body.clear(),
            13 => s.headers.push_str("Connection: close\r\n"),
            14 => {
                s.status = 503;
                s.headers = s
                    .headers
                    .replace("application/json", "application/problem+json");
                s.truncate = true;
            }
            _ => unreachable!(),
        }
        assert!(
            call(&f, &c, s).await.is_err(),
            "reject response case {case}"
        );
    }
    for case in 0..3 {
        let mut l = limits();
        let mut s = response();
        match case {
            0 => {
                l.header_time = Duration::from_millis(100);
                s.header_delay = Duration::from_millis(250);
            }
            1 => {
                l.body_time = Duration::from_millis(100);
                s.body_delay = Duration::from_millis(250);
            }
            _ => {
                l.total_time = Duration::from_millis(100);
                s.body_delay = Duration::from_millis(250);
            }
        }
        assert!(call(&f, &client(&f, l, pin(&f)), s).await.is_err());
    }
    let c = client(
        &f,
        Limits {
            handshake_time: Duration::from_millis(100),
            ..limits()
        },
        pin(&f),
    );
    let (out, _silent) = sockets().await;
    assert!(
        timeout(Duration::from_secs(1), c.request(out, &[7; 32], 1, 2))
            .await
            .unwrap()
            .is_err()
    );
}
#[tokio::test]
async fn cancellation_releases_real_socket_and_shared_budget() {
    let f = Fixture::new();
    let c = Arc::new(client(&f, limits(), pin(&f)));
    let (out, incoming) = sockets().await;
    let server = f.server();
    let (started_tx, started) = oneshot::channel();
    let task = tokio::spawn(async move {
        let stream = server
            .accept(incoming, Duration::from_secs(1))
            .await
            .unwrap();
        let mut r = request(stream).await;
        started_tx.send(()).unwrap();
        let mut byte = [0];
        let result = r.read(&mut byte).await;
        assert!(matches!(result, Ok(0) | Err(_)));
    });
    let shared = c.clone();
    let caller = tokio::spawn(async move { shared.request(out, &[7; 32], 1, 2).await });
    timeout(Duration::from_secs(1), started)
        .await
        .unwrap()
        .unwrap();
    let (extra, mut rejected) = sockets().await;
    assert!(c.request(extra, &[7; 32], 1, 2).await.is_err());
    let mut byte = [0];
    assert_eq!(
        timeout(Duration::from_secs(1), rejected.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0
    );
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
    assert!(call(&f, &c, response()).await.is_ok());
}
