//! Actual mTLS transport only: no registration, token or user authorization.
//! Call `peer_certificate` before every protected action, including reused connections.
use rustls::{
    ClientConfig, RootCertStore, ServerConfig,
    client::Resumption,
    pki_types::{CertificateDer, PrivateKeyDer, ServerName},
    server::WebPkiClientVerifier,
};
use sha2::{Digest, Sha256};
use std::{
    fmt, io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};
use x509_parser::prelude::{FromDer, X509Certificate};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TransportError;
impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("verified service transport unavailable")
    }
}
impl fmt::Debug for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for TransportError {}

fn now() -> Result<u64, TransportError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| TransportError)
}

/// Metadata from the actual peer certificate; not an active registration or permit.
#[derive(Debug)]
pub struct PeerCertificate {
    fingerprint: [u8; 32],
    not_before: u64,
    not_after: u64,
}
impl PeerCertificate {
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub fn not_before(&self) -> u64 {
        self.not_before
    }
    pub fn not_after(&self) -> u64 {
        self.not_after
    }
    fn parse(der: &CertificateDer<'_>) -> Result<Self, TransportError> {
        let (remainder, cert) =
            X509Certificate::from_der(der.as_ref()).map_err(|_| TransportError)?;
        if !remainder.is_empty() {
            return Err(TransportError);
        }
        let not_before =
            u64::try_from(cert.validity().not_before.timestamp()).map_err(|_| TransportError)?;
        let not_after =
            u64::try_from(cert.validity().not_after.timestamp()).map_err(|_| TransportError)?;
        if not_after <= not_before || not_after - not_before > 86400 {
            return Err(TransportError);
        }
        Ok(Self {
            fingerprint: Sha256::digest(der.as_ref()).into(),
            not_before,
            not_after,
        })
    }
    fn valid(&self, time: u64) -> bool {
        time >= self.not_before && time < self.not_after
    }
}
fn own(chain: &[CertificateDer<'_>]) -> Result<PeerCertificate, TransportError> {
    let cert = PeerCertificate::parse(chain.first().ok_or(TransportError)?)?;
    if !cert.valid(now()?) {
        return Err(TransportError);
    }
    Ok(cert)
}
fn roots(anchors: Vec<CertificateDer<'static>>) -> Result<RootCertStore, TransportError> {
    if anchors.is_empty() {
        return Err(TransportError);
    }
    let mut store = RootCertStore::empty();
    for anchor in anchors {
        store.add(anchor).map_err(|_| TransportError)?;
    }
    Ok(store)
}

pub struct Server {
    config: Arc<ServerConfig>,
    own: Arc<PeerCertificate>,
}
impl Server {
    pub fn new(
        chain: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        anchors: Vec<CertificateDer<'static>>,
    ) -> Result<Self, TransportError> {
        let own = Arc::new(own(&chain)?);
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let verifier = WebPkiClientVerifier::builder_with_provider(
            Arc::new(roots(anchors)?),
            provider.clone(),
        )
        .build()
        .map_err(|_| TransportError)?;
        let mut config = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|_| TransportError)?
            .with_client_cert_verifier(verifier)
            .with_single_cert(chain, key)
            .map_err(|_| TransportError)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        config.send_tls13_tickets = 0;
        config.max_early_data_size = 0;
        Ok(Self {
            config: Arc::new(config),
            own,
        })
    }
    /// The caller bounds concurrent handshakes; this function does not accept sockets in a loop.
    pub async fn accept(
        &self,
        socket: TcpStream,
        limit: Duration,
    ) -> Result<VerifiedConnection, TransportError> {
        if limit.is_zero() || !self.own.valid(now()?) {
            return Err(TransportError);
        }
        let stream =
            tokio::time::timeout(limit, TlsAcceptor::from(self.config.clone()).accept(socket))
                .await
                .map_err(|_| TransportError)?
                .map_err(|_| TransportError)?;
        VerifiedConnection::new(stream.into(), self.own.clone())
    }
}
pub struct Client {
    config: Arc<ClientConfig>,
    own: Arc<PeerCertificate>,
}
impl Client {
    pub fn new(
        chain: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        anchors: Vec<CertificateDer<'static>>,
    ) -> Result<Self, TransportError> {
        let own = Arc::new(own(&chain)?);
        let mut config =
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .map_err(|_| TransportError)?
                .with_root_certificates(roots(anchors)?)
                .with_client_auth_cert(chain, key)
                .map_err(|_| TransportError)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        config.resumption = Resumption::disabled();
        config.enable_early_data = false;
        Ok(Self {
            config: Arc::new(config),
            own,
        })
    }
    pub async fn connect(
        &self,
        socket: TcpStream,
        name: ServerName<'static>,
        limit: Duration,
    ) -> Result<VerifiedConnection, TransportError> {
        if limit.is_zero() || !self.own.valid(now()?) {
            return Err(TransportError);
        }
        let stream = tokio::time::timeout(
            limit,
            TlsConnector::from(self.config.clone()).connect(name, socket),
        )
        .await
        .map_err(|_| TransportError)?
        .map_err(|_| TransportError)?;
        VerifiedConnection::new(stream.into(), self.own.clone())
    }
}

pub struct VerifiedConnection {
    stream: TlsStream<TcpStream>,
    peer: PeerCertificate,
    own: Arc<PeerCertificate>,
    last_observed: u64,
    failed: bool,
}
impl VerifiedConnection {
    fn new(
        stream: TlsStream<TcpStream>,
        own: Arc<PeerCertificate>,
    ) -> Result<Self, TransportError> {
        let connection = stream.get_ref().1;
        if connection.alpn_protocol() != Some(b"http/1.1".as_slice()) {
            return Err(TransportError);
        }
        let peer = PeerCertificate::parse(
            connection
                .peer_certificates()
                .and_then(|v| v.first())
                .ok_or(TransportError)?,
        )?;
        let time = now()?;
        let mut result = Self {
            stream,
            peer,
            own,
            last_observed: time,
            failed: false,
        };
        result.peer_certificate()?;
        Ok(result)
    }
    /// Rechecks BOTH leaf certificates with current time. Failure permanently poisons this gate.
    /// Fresh registration and token/operation authorization must follow separately.
    pub fn peer_certificate(&mut self) -> Result<&PeerCertificate, TransportError> {
        let time = match now() {
            Ok(t) => t,
            Err(e) => {
                self.failed = true;
                return Err(e);
            }
        };
        if self.failed
            || time < self.last_observed
            || !self.peer.valid(time)
            || !self.own.valid(time)
        {
            self.failed = true;
            return Err(TransportError);
        }
        self.last_observed = time;
        Ok(&self.peer)
    }
}
impl AsyncRead for VerifiedConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}
impl AsyncWrite for VerifiedConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
