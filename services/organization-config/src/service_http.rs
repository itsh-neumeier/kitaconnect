//! Internal membership data over actual mTLS. No listener or user/MFA authorization.
use crate::{
    FacilityId,
    authenticated_membership::{MembershipReader, ReadError},
    membership::{CurrentMembership, IdentityReference},
};
use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use kitaconnect_service_transport::{ConnectionPeer, Server};
use serde::Deserialize;
use serde_json::json;
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};
use zeroize::Zeroizing;

const PATH: &str = "/internal/v1/membership/read";

#[derive(Clone, Copy, Debug)]
pub struct HttpError;
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("internal membership HTTP operation unavailable")
    }
}
impl std::error::Error for HttpError {}

/// Explicit per-instance budgets; no production defaults.
pub struct Limits {
    pub connections: usize,
    pub requests: usize,
    pub body_bytes: usize,
    pub headers: usize,
    pub header_bytes: usize,
    pub handshake_time: Duration,
    pub header_time: Duration,
    pub request_time: Duration,
    pub connection_time: Duration,
}
impl Limits {
    fn validate(&self) -> Result<(), HttpError> {
        if self.connections == 0
            || self.connections > Semaphore::MAX_PERMITS
            || self.requests == 0
            || self.requests > Semaphore::MAX_PERMITS
            || self.body_bytes == 0
            || self.headers == 0
            || self.header_bytes < 8192
            || [
                self.handshake_time,
                self.header_time,
                self.request_time,
                self.connection_time,
            ]
            .iter()
            .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(HttpError);
        }
        Ok(())
    }
}

/// Construct once and share so connection/request budgets remain global to this server.
pub struct MembershipServer {
    tls: Server,
    reader: MembershipReader,
    limits: Limits,
    connections: Semaphore,
    requests: Semaphore,
}
impl MembershipServer {
    pub fn new(tls: Server, reader: MembershipReader, limits: Limits) -> Result<Self, HttpError> {
        limits.validate()?;
        Ok(Self {
            tls,
            reader,
            connections: Semaphore::new(limits.connections),
            requests: Semaphore::new(limits.requests),
            limits,
        })
    }
    /// Own and drive exactly one socket, including while the handler awaits introspection/SQL.
    pub async fn serve(self: Arc<Self>, socket: TcpStream) -> Result<(), HttpError> {
        let _permit = self.connections.try_acquire().map_err(|_| HttpError)?;
        timeout(self.limits.connection_time, async {
            let stream = self
                .tls
                .accept(socket, self.limits.handshake_time)
                .await
                .map_err(|_| HttpError)?;
            let state = BoundConnection {
                server: self.clone(),
                peer: stream.request_peer(),
            };
            let router = Router::new().fallback(handle).with_state(state);
            let mut builder = hyper::server::conn::http1::Builder::new();
            builder
                .timer(TokioTimer::new())
                .header_read_timeout(self.limits.header_time)
                .max_headers(self.limits.headers)
                .max_buf_size(self.limits.header_bytes)
                .auto_date_header(false);
            builder
                .serve_connection(TokioIo::new(stream), TowerToHyperService::new(router))
                .await
                .map_err(|_| HttpError)
        })
        .await
        .map_err(|_| HttpError)?
    }
}
#[derive(Clone)]
struct BoundConnection {
    server: Arc<MembershipServer>,
    peer: ConnectionPeer,
}

fn problem(status: StatusCode) -> Response {
    let mut response = (
        status,
        Json(json!({
            "type": "about:blank", "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16()
        })),
    )
        .into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
        .headers_mut()
        .insert(header::CONNECTION, HeaderValue::from_static("close"));
    response
}
fn unauthorized(invalid: bool) -> Response {
    let mut response = problem(StatusCode::UNAUTHORIZED);
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(if invalid {
            "Bearer realm=\"service\", error=\"invalid_token\""
        } else {
            "Bearer realm=\"service\""
        }),
    );
    response
}
fn no_cache(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response
}
async fn handle(State(state): State<BoundConnection>, request: Request) -> Response {
    let Ok(_permit) = state.server.requests.try_acquire() else {
        return no_cache(problem(StatusCode::TOO_MANY_REQUESTS));
    };
    no_cache(
        timeout(state.server.limits.request_time, dispatch(&state, request))
            .await
            .unwrap_or_else(|_| problem(StatusCode::SERVICE_UNAVAILABLE)),
    )
}

fn alternative_input(request: &Request) -> bool {
    request.uri().query().is_some()
        || request.uri().scheme().is_some()
        || request.uri().authority().is_some()
        || request.headers().keys().any(|name| {
            let name = name.as_str();
            matches!(
                name,
                "cookie"
                    | "content-encoding"
                    | "proxy-authorization"
                    | "forwarded"
                    | "x-client-cert"
                    | "x-client-certificate"
                    | "x-ssl-client-cert"
                    | "ssl-client-cert"
            ) || name.starts_with("x-forwarded-")
                || name.starts_with("x-ssl-client-")
                || name.starts_with("ssl-client-")
        })
}
fn json_content(request: &Request) -> bool {
    let mut values = request.headers().get_all(header::CONTENT_TYPE).iter();
    let Some(value) = values.next() else {
        return false;
    };
    if values.next().is_some() {
        return false;
    }
    let Some(mime) = value
        .to_str()
        .ok()
        .and_then(|v| v.parse::<mime::Mime>().ok())
    else {
        return false;
    };
    mime.type_() == mime::APPLICATION
        && mime.subtype() == mime::JSON
        && mime
            .params()
            .all(|(k, v)| k == mime::CHARSET && v == mime::UTF_8)
        && mime.params().count() <= 1
}
// None means missing authentication. Malformed or duplicate headers are request errors.
fn token(request: &Request) -> Result<Option<Zeroizing<[u8; 32]>>, HttpError> {
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(HttpError);
    }
    let text = value.to_str().map_err(|_| HttpError)?;
    let (scheme, encoded) = text.split_once(' ').ok_or(HttpError)?;
    if !scheme.eq_ignore_ascii_case("Bearer") || encoded.len() != 43 {
        return Err(HttpError);
    }
    let mut token = Zeroizing::new([0; 32]);
    if URL_SAFE_NO_PAD
        .decode_slice(encoded, token.as_mut())
        .map_err(|_| HttpError)?
        != 32
        || URL_SAFE_NO_PAD.encode(token.as_ref()) != encoded
    {
        return Err(HttpError);
    }
    Ok(Some(token))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Body {
    identity_ref: String,
    facility_id: String,
}
fn identifier(text: &str) -> Result<u128, HttpError> {
    if text.len() > 39
        || !matches!(text.as_bytes().first(), Some(b'1'..=b'9'))
        || !text.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(HttpError);
    }
    text.parse().map_err(|_| HttpError)
}
fn parse(body: &[u8]) -> Result<(IdentityReference, FacilityId), HttpError> {
    // Serde struct deserialization also accepts sequences; this protocol requires an object.
    if body.iter().find(|b| !b.is_ascii_whitespace()) != Some(&b'{') {
        return Err(HttpError);
    }
    let body: Body = serde_json::from_slice(body).map_err(|_| HttpError)?;
    Ok((
        IdentityReference::new(identifier(&body.identity_ref)?).ok_or(HttpError)?,
        FacilityId::new(identifier(&body.facility_id)?).ok_or(HttpError)?,
    ))
}
fn representation(current: Option<CurrentMembership>) -> serde_json::Value {
    json!({"membership": current.map(|m| {
        let policy = m.policy();
        let limits = policy.limits();
        json!({ "identity_ref": m.identity().value().to_string(), "facility_id": m.facility().value().to_string(),
            "membership_version": m.version().to_string(), "policy_version": policy.version().value().to_string(),
            "idle_seconds": limits.idle_seconds(), "absolute_seconds": limits.absolute_seconds(),
            "step_up_seconds": limits.step_up_seconds() })
    })})
}
async fn dispatch(state: &BoundConnection, request: Request) -> Response {
    if request.uri().path() != PATH {
        return problem(StatusCode::NOT_FOUND);
    }
    if request.method() != Method::POST {
        let mut response = problem(StatusCode::METHOD_NOT_ALLOWED);
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("POST"));
        return response;
    }
    if alternative_input(&request) {
        return problem(StatusCode::BAD_REQUEST);
    }
    if !json_content(&request) {
        return problem(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let token = match token(&request) {
        Ok(Some(t)) => t,
        Ok(None) => return unauthorized(false),
        Err(_) => return problem(StatusCode::BAD_REQUEST),
    };
    let body = match to_bytes(request.into_body(), state.server.limits.body_bytes).await {
        Ok(body) => body,
        Err(_) => return problem(StatusCode::PAYLOAD_TOO_LARGE),
    };
    let (identity, facility) = match parse(&body) {
        Ok(ids) => ids,
        Err(_) => return problem(StatusCode::BAD_REQUEST),
    };
    let result = state
        .server
        .reader
        .read(&state.peer, &token, identity, facility)
        .await;
    match result {
        Ok(current) => {
            if state.peer.certificate().is_err() {
                return unauthorized(true);
            }
            Json(representation(current)).into_response()
        }
        Err(ReadError::Rejected) => unauthorized(true),
        Err(ReadError::Unavailable) => problem(StatusCode::SERVICE_UNAVAILABLE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifiers_and_object_fields_are_unambiguous() {
        let valid =
            br#"{"identity_ref":"340282366920938463463374607431768211455","facility_id":"1"}"#;
        assert_eq!(parse(valid).unwrap().0.value(), u128::MAX);
        for invalid in [
            "0",
            "01",
            "-1",
            "+1",
            "1.0",
            "1e1",
            " 1",
            "1 ",
            "١",
            "340282366920938463463374607431768211456",
        ] {
            assert!(identifier(invalid).is_err());
        }
        for invalid in [
            r#"["1","1"]"#,
            r#"null"#,
            r#"{}"#,
            r#"{"identity_ref":1,"facility_id":"1"}"#,
            r#"{"identity_ref":null,"facility_id":"1"}"#,
            r#"{"identity_ref":"1","facility_id":"1","extra":true}"#,
            r#"{"identity_ref":"1","identity_ref":"2","facility_id":"1"}"#,
            r#"{"identity_ref":"1","identity\u005fref":"2","facility_id":"1"}"#,
            r#"{"identity_ref":"1","facility_id":"1"} {}"#,
        ] {
            assert!(parse(invalid.as_bytes()).is_err());
        }
    }
    #[test]
    fn only_single_canonical_authorization_is_accepted() {
        let encoded = URL_SAFE_NO_PAD.encode([42; 32]);
        let request = Request::builder()
            .header(header::AUTHORIZATION, format!("bEaReR {encoded}"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert!(token(&request).unwrap().unwrap().as_ref() == &[42; 32]);
        for value in [
            format!("Bearer  {encoded}"),
            format!("Bearer {encoded}="),
            format!("Basic {encoded}"),
            format!("Bearer {encoded}, {encoded}"),
        ] {
            let request = Request::builder()
                .header(header::AUTHORIZATION, value)
                .body(axum::body::Body::empty())
                .unwrap();
            assert!(token(&request).is_err());
        }
        let request = Request::builder()
            .header(header::AUTHORIZATION, format!("Bearer {encoded}"))
            .header(header::AUTHORIZATION, format!("Bearer {encoded}"))
            .body(axum::body::Body::empty())
            .unwrap();
        assert!(token(&request).is_err());
    }
}
