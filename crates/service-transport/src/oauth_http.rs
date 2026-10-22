//! Bounded OAuth HTTP transport only. A reply is never a current authorization result.
use crate::{Client, RegisteredDnsName};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request,
    body::Bytes,
    http::{HeaderMap, header},
};
use hyper_util::rt::TokioIo;
use rustls::pki_types::ServerName;
use std::{
    fmt,
    time::{Duration, Instant},
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug)]
pub struct HttpError;
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuth HTTP transport unavailable")
    }
}
impl std::error::Error for HttpError {}

#[derive(Clone, Copy, Debug)]
pub enum OAuthEndpoint {
    Token,
    Introspect,
}
impl OAuthEndpoint {
    fn path(self) -> &'static str {
        match self {
            Self::Token => "/oauth/token",
            Self::Introspect => "/oauth/introspect",
        }
    }
}

/// Explicit limits only. No productive policy defaults.
pub struct ClientLimits {
    pub parallel: usize,
    pub form_bytes: usize,
    pub response_bytes: usize,
    pub headers: usize,
    pub header_bytes: usize,
    pub handshake_time: Duration,
    pub header_time: Duration,
    pub body_time: Duration,
    pub total_time: Duration,
}
impl ClientLimits {
    fn validate(&self) -> Result<(), HttpError> {
        if self.parallel == 0
            || self.parallel > Semaphore::MAX_PERMITS
            || self.form_bytes == 0
            || self.response_bytes == 0
            || self.headers == 0
            || self.header_bytes < 8192
            || [
                self.handshake_time,
                self.header_time,
                self.body_time,
                self.total_time,
            ]
            .iter()
            .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(HttpError);
        }
        Ok(())
    }
}

/// Received bytes only. The consumer must validate issuer, audience, scope, time and binding.
pub struct OAuthReply {
    status: u16,
    body: Zeroizing<Vec<u8>>,
    server_certificate: [u8; 32],
}
impl OAuthReply {
    pub fn status(&self) -> u16 {
        self.status
    }
    pub fn body(&self) -> &[u8] {
        &self.body
    }
    /// Observed transport metadata, not proof of current server registration or permission.
    pub fn server_certificate(&self) -> [u8; 32] {
        self.server_certificate
    }
}
impl fmt::Debug for OAuthReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuthReply([REDACTED])")
    }
}

pub struct OAuthHttpClient {
    tls: Client,
    name: RegisteredDnsName,
    limits: ClientLimits,
    active: Semaphore,
}
impl OAuthHttpClient {
    /// Share this client to share the budget. No database, DNS lookup or productive fallback.
    pub fn new(
        tls: Client,
        name: RegisteredDnsName,
        limits: ClientLimits,
    ) -> Result<Self, HttpError> {
        limits.validate()?;
        Ok(Self {
            tls,
            name,
            active: Semaphore::new(limits.parallel),
            limits,
        })
    }
    pub async fn request(
        &self,
        socket: TcpStream,
        endpoint: OAuthEndpoint,
        form: &[u8],
    ) -> Result<OAuthReply, HttpError> {
        let _permit = self.active.try_acquire().map_err(|_| HttpError)?;
        if form.len() > self.limits.form_bytes {
            return Err(HttpError);
        }
        timeout(self.limits.total_time, async {
            let port = socket.peer_addr().map_err(|_| HttpError)?.port();
            let name =
                ServerName::try_from(self.name.as_str().to_owned()).map_err(|_| HttpError)?;
            let stream = self
                .tls
                .connect(socket, name, self.limits.handshake_time)
                .await
                .map_err(|_| HttpError)?;
            let peer = stream.request_peer();
            let mut builder = hyper::client::conn::http1::Builder::new();
            builder
                .max_headers(self.limits.headers)
                .max_buf_size(self.limits.header_bytes);
            let (mut sender, connection) = builder
                .handshake::<_, Full<Bytes>>(TokioIo::new(stream))
                .await
                .map_err(|_| HttpError)?;
            let request = Request::builder()
                .method("POST")
                .uri(endpoint.path())
                .header(header::HOST, format!("{}:{port}", self.name.as_str()))
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::ACCEPT, "application/json")
                .body(Full::new(Bytes::copy_from_slice(form)))
                .map_err(|_| HttpError)?;
            let receive = async {
                let response = timeout(self.limits.header_time, sender.send_request(request))
                    .await
                    .map_err(|_| HttpError)?
                    .map_err(|_| HttpError)?;
                if response.status().is_redirection()
                    || !single(response.headers(), header::CACHE_CONTROL, "no-store")
                    || !single(response.headers(), header::PRAGMA, "no-cache")
                    || response
                        .headers()
                        .get_all(header::CONTENT_TYPE)
                        .iter()
                        .count()
                        > 1
                {
                    return Err(HttpError);
                }
                let json = json_type(response.headers());
                let status = response.status().as_u16();
                peer.certificate().map_err(|_| HttpError)?;
                let body = timeout(
                    self.limits.body_time,
                    Limited::new(response.into_body(), self.limits.response_bytes).collect(),
                )
                .await
                .map_err(|_| HttpError)?
                .map_err(|_| HttpError)?
                .to_bytes();
                if !body.is_empty() && !json {
                    return Err(HttpError);
                }
                let server_certificate = peer.certificate().map_err(|_| HttpError)?.fingerprint();
                Ok(OAuthReply {
                    status,
                    body: Zeroizing::new(body.to_vec()),
                    server_certificate,
                })
            };
            // Drive HTTP in this same future. Completion/cancellation drops the actual stream;
            // no detached connection task, redirect following or reusable positive cache exists.
            tokio::select! {
                result = receive => result,
                _ = connection => Err(HttpError),
            }
        })
        .await
        .map_err(|_| HttpError)?
    }
}

fn single(headers: &HeaderMap, name: header::HeaderName, expected: &str) -> bool {
    let mut values = headers.get_all(name).iter();
    let Some(value) = values.next() else {
        return false;
    };
    values.next().is_none()
        && value
            .to_str()
            .is_ok_and(|v| v.trim().eq_ignore_ascii_case(expected))
}
fn json_type(headers: &HeaderMap) -> bool {
    let mut values = headers.get_all(header::CONTENT_TYPE).iter();
    let Some(value) = values.next() else {
        return false;
    };
    if values.next().is_some() {
        return false;
    }
    let Some(value) = value
        .to_str()
        .ok()
        .and_then(|s| s.parse::<mime::Mime>().ok())
    else {
        return false;
    };
    value.type_() == mime::APPLICATION
        && value.subtype() == mime::JSON
        && value
            .params()
            .all(|(k, v)| k == mime::CHARSET && v == mime::UTF_8)
        && value.params().count() <= 1
}
