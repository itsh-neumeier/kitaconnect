//! Current composed data only. No session issuance, domain rights or lossy port conversion.
use crate::{
    IdentityId, login::LoginFacilityId, organization_http::MembershipReader,
    rights_assurance::Reader as RightsReader,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kitaconnect_service_oauth::{ResponseLimits, TokenResponse};
use kitaconnect_service_transport::oauth_http::{OAuthEndpoint, OAuthHttpClient};
use std::{
    fmt,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};
use zeroize::Zeroizing;

const TOKEN_FORM: &[u8] =
    b"client_id=identity-access&grant_type=client_credentials&scope=membership%3Aread";

#[derive(Clone, Copy)]
pub struct ReadError;
impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("current login source unavailable")
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for ReadError {}

#[derive(Clone, Copy)]
pub struct ReadLimits {
    pub parallel: usize,
    pub response_bytes: usize,
    pub connect_time: Duration,
    pub total_time: Duration,
}
impl ReadLimits {
    fn validate(self) -> Result<(), ReadError> {
        if self.parallel == 0
            || self.parallel > Semaphore::MAX_PERMITS
            || self.response_bytes == 0
            || self.connect_time > self.total_time
            || [self.connect_time, self.total_time]
                .iter()
                .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(ReadError);
        }
        Ok(())
    }
}
pub struct Configuration {
    pub address: SocketAddr,
    pub server_certificate: [u8; 32],
    pub limits: ReadLimits,
}
pub struct CurrentLogin {
    identity: IdentityId,
    facility: LoginFacilityId,
    membership_version: u64,
    policy_version: u64,
    rights_version: u64,
    idle_seconds: u64,
    absolute_seconds: u64,
    step_up_seconds: u64,
    mfa_required: bool,
}
impl fmt::Debug for CurrentLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CurrentLogin(REDACTED)")
    }
}
impl CurrentLogin {
    pub fn identity(&self) -> IdentityId {
        self.identity
    }
    pub fn facility(&self) -> LoginFacilityId {
        self.facility
    }
    pub fn membership_version(&self) -> u64 {
        self.membership_version
    }
    pub fn policy_version(&self) -> u64 {
        self.policy_version
    }
    pub fn rights_version(&self) -> u64 {
        self.rights_version
    }
    pub fn idle_seconds(&self) -> u64 {
        self.idle_seconds
    }
    pub fn absolute_seconds(&self) -> u64 {
        self.absolute_seconds
    }
    pub fn step_up_seconds(&self) -> u64 {
        self.step_up_seconds
    }
    pub fn mfa_required(&self) -> bool {
        self.mfa_required
    }
}
pub struct Reader {
    issuer: Arc<OAuthHttpClient>,
    membership: Arc<MembershipReader>,
    rights: Arc<RightsReader>,
    configuration: Configuration,
    response_limits: ResponseLimits,
    active: Semaphore,
}
impl crate::login::OrganizationAuthority for Reader {
    async fn current(
        &mut self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<crate::login::OrganizationContext, crate::login::LoginError> {
        let current = self
            .read(identity, facility)
            .await
            .map_err(|_| crate::login::LoginError)?;
        crate::login::OrganizationContext::new(
            identity,
            facility,
            current.policy_version,
            current.idle_seconds,
            current.absolute_seconds,
            current.step_up_seconds,
            current.mfa_required,
        )?
        .with_source_versions(current.membership_version, current.rights_version)
    }
}
fn timely(elapsed: Duration, seconds: u64) -> Result<(), ReadError> {
    if !(1..=300).contains(&seconds) || elapsed >= Duration::from_secs(seconds) {
        return Err(ReadError);
    }
    Ok(())
}
impl Reader {
    pub fn new(
        issuer: Arc<OAuthHttpClient>,
        membership: Arc<MembershipReader>,
        rights: Arc<RightsReader>,
        configuration: Configuration,
    ) -> Result<Self, ReadError> {
        configuration.limits.validate()?;
        let response_limits =
            ResponseLimits::new(configuration.limits.response_bytes).map_err(|_| ReadError)?;
        let active = Semaphore::new(configuration.limits.parallel);
        Ok(Self {
            issuer,
            membership,
            rights,
            configuration,
            response_limits,
            active,
        })
    }
    /// Fresh invocation of every source. Never accepts a user-supplied service token.
    pub async fn read(
        &self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<CurrentLogin, ReadError> {
        let _permit = self.active.try_acquire().map_err(|_| ReadError)?;
        timeout(
            self.configuration.limits.total_time,
            self.read_inner(identity, facility),
        )
        .await
        .map_err(|_| ReadError)?
    }
    async fn read_inner(
        &self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<CurrentLogin, ReadError> {
        let started = Instant::now();
        let socket = timeout(
            self.configuration.limits.connect_time,
            TcpStream::connect(self.configuration.address),
        )
        .await
        .map_err(|_| ReadError)?
        .map_err(|_| ReadError)?;
        let response = self
            .issuer
            .request(socket, OAuthEndpoint::Token, TOKEN_FORM)
            .await
            .map_err(|_| ReadError)?;
        if response.server_certificate() != self.configuration.server_certificate {
            return Err(ReadError);
        }
        let issued = TokenResponse::parse(
            response.status(),
            response.body(),
            "membership:read",
            self.response_limits,
        )
        .map_err(|_| ReadError)?;
        let mut token = Zeroizing::new([0; 32]);
        if URL_SAFE_NO_PAD
            .decode_slice(issued.expose_token(), token.as_mut())
            .map_err(|_| ReadError)?
            != 32
        {
            return Err(ReadError);
        }
        timely(started.elapsed(), issued.expires_in())?;
        let member = self
            .membership
            .read(&token, identity, facility)
            .await
            .map_err(|_| ReadError)?
            .ok_or(ReadError)?;
        let rights = self
            .rights
            .read(identity, facility)
            .await
            .map_err(|_| ReadError)?;
        timely(started.elapsed(), issued.expires_in())?;
        if member.identity() != identity
            || member.facility() != facility
            || rights.identity() != identity
            || rights.facility() != facility
        {
            return Err(ReadError);
        }
        Ok(CurrentLogin {
            identity,
            facility,
            membership_version: member.membership_version(),
            policy_version: member.policy_version(),
            rights_version: rights.version(),
            idle_seconds: member.idle_seconds(),
            absolute_seconds: member.absolute_seconds(),
            step_up_seconds: member.step_up_seconds(),
            mfa_required: rights.mfa_required(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_outer_limits_are_explicit() {
        let valid = ReadLimits {
            parallel: 1,
            response_bytes: 2048,
            connect_time: Duration::from_secs(1),
            total_time: Duration::from_secs(2),
        };
        assert!(valid.validate().is_ok());
        for parallel in [0, Semaphore::MAX_PERMITS + 1] {
            assert!(ReadLimits { parallel, ..valid }.validate().is_err());
        }
        assert!(
            ReadLimits {
                response_bytes: 0,
                ..valid
            }
            .validate()
            .is_err()
        );
        for value in [Duration::ZERO, Duration::MAX] {
            assert!(
                ReadLimits {
                    connect_time: value,
                    ..valid
                }
                .validate()
                .is_err()
            );
            assert!(
                ReadLimits {
                    total_time: value,
                    ..valid
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            ReadLimits {
                connect_time: Duration::from_secs(3),
                ..valid
            }
            .validate()
            .is_err()
        );
    }
    #[test]
    fn conservative_token_window_rejects_expiry_and_invalid_lifetimes() {
        for seconds in [1, 300] {
            assert!(timely(Duration::ZERO, seconds).is_ok());
            assert!(
                timely(
                    Duration::from_secs(seconds) - Duration::from_nanos(1),
                    seconds
                )
                .is_ok()
            );
            assert!(timely(Duration::from_secs(seconds), seconds).is_err());
            assert!(timely(Duration::MAX, seconds).is_err());
        }
        for seconds in [0, 301, u64::MAX] {
            assert!(timely(Duration::ZERO, seconds).is_err());
        }
    }
    #[test]
    fn output_preserves_distinct_versions_and_redacts_diagnostics() {
        let value = CurrentLogin {
            identity: IdentityId::new(1).unwrap(),
            facility: LoginFacilityId::new(2).unwrap(),
            membership_version: 3,
            policy_version: 4,
            rights_version: 5,
            idle_seconds: 900,
            absolute_seconds: 28800,
            step_up_seconds: 300,
            mfa_required: true,
        };
        assert_eq!(
            (
                value.membership_version(),
                value.policy_version(),
                value.rights_version()
            ),
            (3, 4, 5)
        );
        assert_eq!(format!("{value:?}"), "CurrentLogin(REDACTED)");
        assert_eq!(format!("{ReadError:?}"), "current login source unavailable");
    }
}
