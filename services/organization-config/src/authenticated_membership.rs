//! Current service-authenticated membership data only, never user/MFA permission.
//! No listener, foreign database, caller transaction, role switch or positive cache.
use crate::{
    FacilityId,
    membership::{self, CurrentMembership, IdentityReference},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kitaconnect_service_oauth::{ExpectedActive, IntrospectionResponse, ResponseLimits};
use kitaconnect_service_transport::{
    ConnectionPeer, RegisteredDnsName,
    oauth_http::{OAuthEndpoint, OAuthHttpClient, OAuthReply},
};
use sqlx::{Connection, PgConnection, postgres::PgConnectOptions};
use std::{
    fmt,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    Rejected,
    Unavailable,
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rejected => "membership request rejected",
            Self::Unavailable => "membership check unavailable",
        })
    }
}
impl std::error::Error for ReadError {}
impl From<sqlx::Error> for ReadError {
    fn from(_: sqlx::Error) -> Self {
        Self::Unavailable
    }
}

#[derive(Clone, Copy)]
pub struct ReadLimits {
    pub parallel: usize,
    pub response_bytes: usize,
    pub connect_time: Duration,
    pub database_time: Duration,
    pub total_time: Duration,
}
pub struct Configuration {
    pub address: SocketAddr,
    pub issuer: String,
    pub caller_name: RegisteredDnsName,
    /// Explicit trusted configuration, not a claim received in the OAuth reply.
    pub server_certificate: [u8; 32],
    pub limits: ReadLimits,
}
pub struct MembershipReader {
    client: OAuthHttpClient,
    configuration: Configuration,
    database: PgConnectOptions,
    budget: Semaphore,
    response_limits: ResponseLimits,
}
fn database_timeout(duration: Duration) -> Result<String, ReadError> {
    let ms = duration.as_millis() + u128::from(!duration.subsec_nanos().is_multiple_of(1_000_000));
    if !(1..=i32::MAX as u128).contains(&ms) {
        return Err(ReadError::Unavailable);
    }
    Ok(ms.to_string())
}
impl MembershipReader {
    pub fn new(
        client: OAuthHttpClient,
        mut configuration: Configuration,
        database: PgConnectOptions,
    ) -> Result<Self, ReadError> {
        let limits = configuration.limits;
        if limits.parallel == 0
            || limits.parallel > Semaphore::MAX_PERMITS
            || [limits.connect_time, limits.database_time, limits.total_time]
                .iter()
                .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
            || limits.connect_time > limits.total_time
            || limits.database_time > limits.total_time
        {
            return Err(ReadError::Unavailable);
        }
        let issuer = url::Url::parse(&configuration.issuer).map_err(|_| ReadError::Unavailable)?;
        if configuration.issuer.len() > 2048
            || issuer.scheme() != "https"
            || issuer.host_str().is_none()
            || !issuer.username().is_empty()
            || issuer.password().is_some()
            || issuer.query().is_some()
            || issuer.fragment().is_some()
        {
            return Err(ReadError::Unavailable);
        }
        configuration.issuer = issuer.to_string();
        let response_limits =
            ResponseLimits::new(limits.response_bytes).map_err(|_| ReadError::Unavailable)?;
        let ms = database_timeout(limits.database_time)?;
        let database = database
            .application_name("kc_authenticated_membership_request")
            .options([
                ("statement_timeout", ms.as_str()),
                ("transaction_timeout", ms.as_str()),
                ("idle_in_transaction_session_timeout", ms.as_str()),
            ]);
        Ok(Self {
            client,
            configuration,
            database,
            budget: Semaphore::new(limits.parallel),
            response_limits,
        })
    }
    /// Caller owns/drives the actual incoming connection and supplies a previously checked user reference.
    /// A result is a data snapshot only; it cannot establish login assurance or object rights.
    pub async fn read(
        &self,
        peer: &ConnectionPeer,
        token: &[u8; 32],
        identity: IdentityReference,
        facility: FacilityId,
    ) -> Result<Option<CurrentMembership>, ReadError> {
        let _permit = self
            .budget
            .try_acquire()
            .map_err(|_| ReadError::Unavailable)?;
        timeout(
            self.configuration.limits.total_time,
            self.read_inner(peer, token, identity, facility),
        )
        .await
        .map_err(|_| ReadError::Unavailable)?
    }
    fn peer(&self, peer: &ConnectionPeer) -> Result<[u8; 32], ReadError> {
        let certificate = peer.certificate().map_err(|_| ReadError::Rejected)?;
        if !certificate.matches_dns_name(&self.configuration.caller_name) {
            return Err(ReadError::Rejected);
        }
        Ok(certificate.fingerprint())
    }
    fn validate(
        &self,
        reply: &OAuthReply,
        certificate: [u8; 32],
        now: u64,
    ) -> Result<(), ReadError> {
        if reply.server_certificate() != self.configuration.server_certificate {
            return Err(ReadError::Rejected);
        }
        if reply.status() != 200 {
            return Err(if matches!(reply.status(), 400 | 401) {
                ReadError::Rejected
            } else {
                ReadError::Unavailable
            });
        }
        let expected = ExpectedActive {
            issuer: &self.configuration.issuer,
            client: "identity-access",
            target: "organization-config",
            scope: "membership:read",
            certificate,
            now,
        };
        match IntrospectionResponse::parse(
            reply.status(),
            reply.body(),
            expected,
            self.response_limits,
        )
        .map_err(|_| ReadError::Unavailable)?
        {
            IntrospectionResponse::Active(_) => Ok(()),
            IntrospectionResponse::Inactive => Err(ReadError::Rejected),
        }
    }
    async fn read_inner(
        &self,
        peer: &ConnectionPeer,
        token: &[u8; 32],
        identity: IdentityReference,
        facility: FacilityId,
    ) -> Result<Option<CurrentMembership>, ReadError> {
        let certificate = self.peer(peer)?;
        let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(token));
        let form = Zeroizing::new(format!(
            "client_id=organization-config&token={}",
            encoded.as_str()
        ));
        let socket = timeout(
            self.configuration.limits.connect_time,
            TcpStream::connect(self.configuration.address),
        )
        .await
        .map_err(|_| ReadError::Unavailable)?
        .map_err(|_| ReadError::Unavailable)?;
        let reply = self
            .client
            .request(socket, OAuthEndpoint::Introspect, form.as_bytes())
            .await
            .map_err(|_| ReadError::Unavailable)?;
        self.peer(peer)?;
        // Never open a local data connection for an obvious transport/status/pin failure.
        if reply.server_certificate() != self.configuration.server_certificate {
            return Err(ReadError::Rejected);
        }
        if reply.status() != 200 {
            return Err(if matches!(reply.status(), 400 | 401) {
                ReadError::Rejected
            } else {
                ReadError::Unavailable
            });
        }
        let mut database = PgConnection::connect_with(&self.database).await?;
        let before = clock(&mut database).await?;
        self.validate(&reply, certificate, before)?;
        self.peer(peer)?;
        let result = membership::read_current(&mut database, identity, facility)
            .await
            .map_err(|_| ReadError::Unavailable)?;
        let after = clock(&mut database).await?;
        if after < before {
            return Err(ReadError::Unavailable);
        }
        self.validate(&reply, certificate, after)?;
        self.peer(peer)?;
        Ok(result)
    }
}
async fn clock(database: &mut PgConnection) -> Result<u64, ReadError> {
    let now: i64 =
        sqlx::query_scalar("SELECT floor(extract(epoch FROM clock_timestamp()))::bigint")
            .fetch_one(database)
            .await?;
    u64::try_from(now).map_err(|_| ReadError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn database_timeouts_cannot_disable_cleanup() {
        assert!(database_timeout(Duration::ZERO).is_err());
        assert!(database_timeout(Duration::from_millis(i32::MAX as u64 + 1)).is_err());
        assert_eq!(database_timeout(Duration::from_nanos(1)).unwrap(), "1");
        assert_eq!(database_timeout(Duration::from_millis(17)).unwrap(), "17");
    }
}
