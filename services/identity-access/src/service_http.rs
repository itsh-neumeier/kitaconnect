//! Bounded internal OAuth over actual mTLS. No listener, production defaults or role changes.
use crate::{
    service_auth::{self, RegisteredNames, ServiceAuthenticationError},
    service_oauth::{self, FormLimits, IntrospectionRequest, OAuthError, TokenRequest},
    service_tokens::{self, AuthorityConfiguration, ServiceName, ServiceScope},
};
use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use kitaconnect_service_transport::{ConnectionPeer, Server};
use serde_json::json;
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};
use std::{fmt, sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};

#[derive(Clone, Copy, Debug)]
pub struct HttpError;
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("internal HTTP operation unavailable")
    }
}
impl std::error::Error for HttpError {}

/// Every value is explicitly configured. No Default implementation or productive policy.
pub struct Limits {
    pub connections: usize,
    pub requests: usize,
    pub body_bytes: usize,
    pub parameters: usize,
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
            || self.parameters == 0
            || self.headers == 0
            || self.header_bytes < 8192
            || [
                self.handshake_time,
                self.header_time,
                self.request_time,
                self.connection_time,
            ]
            .iter()
            .any(Duration::is_zero)
        {
            return Err(HttpError);
        }
        Ok(())
    }
}

/// Own this once per server, then share it. Creating separate instances creates separate budgets.
pub struct OAuthServer {
    tls: Server,
    config: AuthorityConfiguration,
    names: RegisteredNames,
    database: PgConnectOptions,
    limits: Limits,
    connections: Semaphore,
    requests: Semaphore,
    token_seconds: u16,
}
impl OAuthServer {
    pub fn new(
        tls: Server,
        config: AuthorityConfiguration,
        names: RegisteredNames,
        database: PgConnectOptions,
        limits: Limits,
        token_seconds: u16,
    ) -> Result<Self, HttpError> {
        limits.validate()?;
        if !(1..=300).contains(&token_seconds) {
            return Err(HttpError);
        }
        Ok(Self {
            tls,
            config,
            names,
            database,
            connections: Semaphore::new(limits.connections),
            requests: Semaphore::new(limits.requests),
            limits,
            token_seconds,
        })
    }

    /// A single owned socket only. Excess connections close before TLS, without plaintext HTTP.
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
            // State is created here, never extracted from request headers or claims.
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
    server: Arc<OAuthServer>,
    peer: ConnectionPeer,
}

fn empty(status: StatusCode) -> Response {
    status.into_response()
}
fn error(status: StatusCode, code: OAuthError) -> Response {
    (status, Json(code.response())).into_response()
}
fn unavailable() -> Response {
    empty(StatusCode::SERVICE_UNAVAILABLE)
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
        return no_cache(empty(StatusCode::TOO_MANY_REQUESTS));
    };
    let response = timeout(state.server.limits.request_time, dispatch(&state, request))
        .await
        .unwrap_or_else(|_| unavailable());
    no_cache(response)
}

fn auth_header(request: &Request) -> Result<Option<String>, OAuthError> {
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(OAuthError::InvalidRequest);
    }
    let text = value.to_str().map_err(|_| OAuthError::InvalidRequest)?;
    let scheme = text.split(' ').next().unwrap_or_default();
    if scheme.is_empty()
        || scheme.len() > 32
        || !scheme
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
    {
        return Err(OAuthError::InvalidRequest);
    }
    Ok(Some(format!("{scheme} realm=\"service\"")))
}
fn form_content(request: &Request) -> bool {
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
        && mime.subtype() == mime::WWW_FORM_URLENCODED
        && mime
            .params()
            .all(|(name, value)| name == mime::CHARSET && value == mime::UTF_8)
        && mime.params().count() <= 1
}

async fn dispatch(state: &BoundConnection, request: Request) -> Response {
    let introspect = match request.uri().path() {
        "/oauth/token" => false,
        "/oauth/introspect" => true,
        _ => return empty(StatusCode::NOT_FOUND),
    };
    if request.method() != Method::POST {
        let mut response = empty(StatusCode::METHOD_NOT_ALLOWED);
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("POST"));
        return response;
    }
    if request.uri().query().is_some() || request.headers().contains_key(header::COOKIE) {
        return error(StatusCode::BAD_REQUEST, OAuthError::InvalidRequest);
    }
    match auth_header(&request) {
        Err(code) => return error(StatusCode::BAD_REQUEST, code),
        Ok(Some(challenge)) => {
            let mut response = error(StatusCode::UNAUTHORIZED, OAuthError::InvalidClient);
            let Ok(challenge) = HeaderValue::from_str(&challenge) else {
                return unavailable();
            };
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, challenge);
            return response;
        }
        Ok(None) => (),
    }
    if !form_content(&request) {
        return empty(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let body = match to_bytes(request.into_body(), state.server.limits.body_bytes).await {
        Ok(body) => body,
        Err(_) => return empty(StatusCode::PAYLOAD_TOO_LARGE),
    };
    let limits = FormLimits::new(
        state.server.limits.body_bytes,
        state.server.limits.parameters,
    )
    .expect("validated positive limits");
    // These connections are owned by this future. Cancellation drops the physical connection;
    // an unfinished transaction cannot return to a reusable pool.
    if introspect {
        match IntrospectionRequest::parse(&body, limits) {
            Ok(parsed) => inspect(state, parsed).await,
            Err(code) => error(
                if code == OAuthError::InvalidClient {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::BAD_REQUEST
                },
                code,
            ),
        }
    } else {
        match TokenRequest::parse(&body, limits) {
            Ok(parsed) => issue(state, parsed).await,
            Err(code) => error(StatusCode::BAD_REQUEST, code),
        }
    }
}
enum Rejection {
    Unavailable,
    OAuth(StatusCode, OAuthError),
}
impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        match self {
            Self::Unavailable => unavailable(),
            Self::OAuth(status, code) => error(status, code),
        }
    }
}
async fn authenticate(
    state: &BoundConnection,
    c: &mut PgConnection,
    claimed: ServiceName,
    denied: StatusCode,
) -> Result<service_tokens::PeerReference, Rejection> {
    service_auth::authenticate(
        c,
        &state.server.config,
        &state.server.names,
        claimed,
        &state.peer,
    )
    .await
    .map_err(|e| match e {
        ServiceAuthenticationError::Rejected => Rejection::OAuth(denied, OAuthError::InvalidClient),
        ServiceAuthenticationError::Unavailable => Rejection::Unavailable,
    })
}
async fn grant(
    c: &mut PgConnection,
    client: ServiceName,
    target: ServiceName,
    scope: ServiceScope,
) -> Result<(), Rejection> {
    let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT FROM identity_access.service_grants g JOIN identity_access.service_clients c ON c.name=g.client JOIN identity_access.service_clients t ON t.name=g.target WHERE g.client=$1 AND g.target=$2 AND g.scope=$3 AND g.active AND c.active AND t.active)")
        .bind(client.text()).bind(target.text()).bind(scope.text()).fetch_one(c).await.map_err(|_| Rejection::Unavailable)?;
    if allowed {
        Ok(())
    } else {
        Err(Rejection::OAuth(
            StatusCode::BAD_REQUEST,
            OAuthError::InvalidScope,
        ))
    }
}
async fn issue(state: &BoundConnection, request: TokenRequest) -> Response {
    let Ok(mut c) = PgConnection::connect_with(&state.server.database).await else {
        return unavailable();
    };
    let client = request.claimed_client();
    let peer = match authenticate(state, &mut c, client, StatusCode::BAD_REQUEST).await {
        Ok(peer) => peer,
        Err(response) => return response.into_response(),
    };
    if let Err(response) = grant(&mut c, client, request.target(), request.scope()).await {
        return response.into_response();
    }
    let Ok(mut issued) = service_tokens::issue_with_metadata(
        &mut c,
        &state.server.config,
        peer,
        request.target(),
        request.scope(),
        state.server.token_seconds,
    )
    .await
    else {
        return unavailable();
    };
    if let Err(response) = authenticate(state, &mut c, client, StatusCode::BAD_REQUEST).await {
        return response.into_response();
    }
    if let Err(response) = grant(&mut c, client, request.target(), request.scope()).await {
        return response.into_response();
    }
    let Ok(remaining) = service_tokens::remaining_lifetime(&mut c, &mut issued).await else {
        return unavailable();
    };
    if state.peer.certificate().is_err() {
        return unavailable();
    }
    let encoded = service_oauth::encode_token(issued.token());
    Json(
        json!({"access_token": encoded.as_str(), "token_type":"Bearer",
        "scope": issued.scope().text(), "expires_in": remaining}),
    )
    .into_response()
}
async fn inspect(state: &BoundConnection, request: IntrospectionRequest) -> Response {
    let Ok(mut c) = PgConnection::connect_with(&state.server.database).await else {
        return unavailable();
    };
    let peer = match authenticate(
        state,
        &mut c,
        request.claimed_client(),
        StatusCode::UNAUTHORIZED,
    )
    .await
    {
        Ok(peer) => peer,
        Err(response) => return response.into_response(),
    };
    if peer.service() == ServiceName::Gateway {
        return error(StatusCode::UNAUTHORIZED, OAuthError::InvalidClient);
    }
    let active = if let Some(token) = request.token() {
        match service_tokens::introspect(&mut c, &state.server.config, peer, token).await {
            Ok(active) => active,
            Err(_) => return unavailable(),
        }
    } else {
        None
    };
    // Refresh authentication after database waits, including negative/malformed token responses.
    if let Err(response) = authenticate(
        state,
        &mut c,
        request.claimed_client(),
        StatusCode::UNAUTHORIZED,
    )
    .await
    {
        return response.into_response();
    }
    if state.peer.certificate().is_err() {
        return unavailable();
    }
    Json(service_oauth::introspection_response(active.as_ref())).into_response()
}
