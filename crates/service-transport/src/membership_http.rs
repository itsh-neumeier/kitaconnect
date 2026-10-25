//! Fixed membership HTTP transport. A reply is a bounded message, never permission.
use crate::{Client, RegisteredDnsName};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request,
    body::Bytes,
    http::{HeaderMap, HeaderValue, header},
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
        f.write_str("membership HTTP transport unavailable")
    }
}
impl std::error::Error for HttpError {}

/// Explicit limits; no productive defaults.
#[derive(Clone, Copy)]
pub struct ClientLimits {
    pub parallel: usize,
    pub request_bytes: usize,
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
            || self.request_bytes == 0
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
pub struct MembershipReply {
    status: u16,
    body: Zeroizing<Vec<u8>>,
}
impl MembershipReply {
    pub fn status(&self) -> u16 {
        self.status
    }
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}
impl fmt::Debug for MembershipReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MembershipReply([REDACTED])")
    }
}
pub struct MembershipHttpClient {
    tls: Client,
    name: RegisteredDnsName,
    server_certificate: [u8; 32],
    limits: ClientLimits,
    active: Semaphore,
}
impl MembershipHttpClient {
    /// Share this instance to share its budget. The pin is trusted explicit configuration.
    pub fn new(
        tls: Client,
        name: RegisteredDnsName,
        server_certificate: [u8; 32],
        limits: ClientLimits,
    ) -> Result<Self, HttpError> {
        limits.validate()?;
        Ok(Self {
            tls,
            name,
            server_certificate,
            active: Semaphore::new(limits.parallel),
            limits,
        })
    }
    pub async fn request(
        &self,
        socket: TcpStream,
        token: &[u8; 32],
        identity: u128,
        facility: u128,
    ) -> Result<MembershipReply, HttpError> {
        let _permit = self.active.try_acquire().map_err(|_| HttpError)?;
        if identity == 0 || facility == 0 {
            return Err(HttpError);
        }
        let body = format!(r#"{{"identity_ref":"{identity}","facility_id":"{facility}"}}"#);
        if body.len() > self.limits.request_bytes {
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
            // Check the configured server before creating or sending the credential header.
            if peer.certificate().map_err(|_| HttpError)?.fingerprint() != self.server_certificate {
                return Err(HttpError);
            }
            let mut builder = hyper::client::conn::http1::Builder::new();
            builder
                .max_headers(self.limits.headers)
                .max_buf_size(self.limits.header_bytes);
            let (mut sender, connection) = builder
                .handshake::<_, Full<Bytes>>(TokioIo::new(stream))
                .await
                .map_err(|_| HttpError)?;
            let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(token));
            let authorization = Zeroizing::new(format!("Bearer {}", encoded.as_str()));
            let mut secret =
                HeaderValue::from_str(authorization.as_str()).map_err(|_| HttpError)?;
            secret.set_sensitive(true);
            let request = Request::builder()
                .method("POST")
                .uri("/internal/v1/membership/read")
                .header(header::HOST, format!("{}:{port}", self.name.as_str()))
                .header(header::AUTHORIZATION, secret)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, application/problem+json")
                .body(Full::new(Bytes::from(body)))
                .map_err(|_| HttpError)?;
            let receive = async {
                let response = timeout(self.limits.header_time, sender.send_request(request))
                    .await
                    .map_err(|_| HttpError)?
                    .map_err(|_| HttpError)?;
                let status = response.status().as_u16();
                if response.status().is_redirection()
                    || (status == 200
                        && response
                            .headers()
                            .get_all(header::CONNECTION)
                            .iter()
                            .any(|value| {
                                value.to_str().map_or(true, |text| {
                                    text.split(',')
                                        .any(|part| part.trim().eq_ignore_ascii_case("close"))
                                })
                            }))
                    || !single(response.headers(), header::CACHE_CONTROL, "no-store")
                    || !single(response.headers(), header::PRAGMA, "no-cache")
                    || response.headers().contains_key(header::CONTENT_ENCODING)
                    || response.headers().contains_key(header::SET_COOKIE)
                    || !media_type(
                        response.headers(),
                        if status == 200 {
                            "application/json"
                        } else {
                            "application/problem+json"
                        },
                    )
                {
                    return Err(HttpError);
                }
                if status == 200 {
                    peer.certificate().map_err(|_| HttpError)?;
                }
                let body = timeout(
                    self.limits.body_time,
                    Limited::new(response.into_body(), self.limits.response_bytes).collect(),
                )
                .await
                .map_err(|_| HttpError)?
                .map_err(|_| HttpError)?
                .to_bytes();
                if body.is_empty() {
                    return Err(HttpError);
                }
                if status == 200 {
                    peer.certificate().map_err(|_| HttpError)?;
                }
                Ok(MembershipReply {
                    status,
                    body: Zeroizing::new(body.to_vec()),
                })
            };
            // No detached driver, retries, redirects or reusable positive results.
            tokio::pin!(receive);
            tokio::pin!(connection);
            tokio::select! {
                result = &mut receive => result,
                _ = &mut connection => match receive.await {
                    // A complete denial remains a denial after the router's deliberate close.
                    Ok(reply) if reply.status != 200 => Ok(reply),
                    _ => Err(HttpError),
                }
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
fn media_type(headers: &HeaderMap, expected: &str) -> bool {
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
    value.essence_str().eq_ignore_ascii_case(expected)
        && value
            .params()
            .all(|(k, v)| k == mime::CHARSET && v == mime::UTF_8)
        && value.params().count() <= 1
}
