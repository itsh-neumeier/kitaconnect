//! Internal state only. PeerReference holds trusted-caller metadata, not TLS evidence.
//! Do not construct peers from browser fields or expose these functions before mTLS exists.
use crate::token::SessionToken;
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection, Row};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ServiceTokenError;
impl fmt::Display for ServiceTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("service token operation unavailable or unsuccessful")
    }
}
impl fmt::Debug for ServiceTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for ServiceTokenError {}
impl From<sqlx::Error> for ServiceTokenError {
    fn from(_: sqlx::Error) -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceName {
    IdentityAccess,
    OrganizationConfig,
    Gateway,
}
impl ServiceName {
    pub fn text(self) -> &'static str {
        match self {
            Self::IdentityAccess => "identity-access",
            Self::OrganizationConfig => "organization-config",
            Self::Gateway => "gateway",
        }
    }
    pub(crate) fn parse(text: &str) -> Result<Self, ServiceTokenError> {
        match text {
            "identity-access" => Ok(Self::IdentityAccess),
            "organization-config" => Ok(Self::OrganizationConfig),
            "gateway" => Ok(Self::Gateway),
            _ => Err(ServiceTokenError),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceScope {
    MembershipRead,
    LocalLogin,
    SessionCheck,
}
impl ServiceScope {
    pub fn text(self) -> &'static str {
        match self {
            Self::MembershipRead => "membership:read",
            Self::LocalLogin => "login:local",
            Self::SessionCheck => "session:check",
        }
    }
    pub(crate) fn parse(text: &str) -> Result<Self, ServiceTokenError> {
        match text {
            "membership:read" => Ok(Self::MembershipRead),
            "login:local" => Ok(Self::LocalLogin),
            "session:check" => Ok(Self::SessionCheck),
            _ => Err(ServiceTokenError),
        }
    }
}
#[derive(Clone, Debug)]
pub struct AuthorityConfiguration {
    installation: [u8; 16],
    issuer: String,
}
impl AuthorityConfiguration {
    pub fn new(installation: [u8; 16], issuer: &str) -> Result<Self, ServiceTokenError> {
        if installation == [0; 16] || issuer.len() > 2048 {
            return Err(ServiceTokenError);
        }
        let parsed = url::Url::parse(issuer).map_err(|_| ServiceTokenError)?;
        if parsed.scheme() != "https"
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || parsed.as_str().len() > 2048
        {
            return Err(ServiceTokenError);
        }
        Ok(Self {
            installation,
            issuer: parsed.to_string(),
        })
    }
    pub fn installation(&self) -> &[u8; 16] {
        &self.installation
    }
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
}
/// Metadata supplied by a trusted caller. This constructor does not authenticate a peer.
#[derive(Clone, Copy, Debug)]
pub struct PeerReference {
    service: ServiceName,
    certificate: [u8; 32],
}
impl PeerReference {
    pub fn new(service: ServiceName, certificate: [u8; 32]) -> Self {
        Self {
            service,
            certificate,
        }
    }
    pub fn service(self) -> ServiceName {
        self.service
    }
    pub fn certificate(self) -> [u8; 32] {
        self.certificate
    }
}
pub struct ServiceToken(SessionToken);
impl ServiceToken {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        SessionToken::from_bytes(bytes).map(Self)
    }
    pub fn expose(&self) -> &[u8; 32] {
        self.0.expose()
    }
    fn generate() -> Result<Self, ServiceTokenError> {
        SessionToken::generate()
            .map(Self)
            .map_err(|_| ServiceTokenError)
    }
    fn fingerprint(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"KitaConnect/service-token/v1");
        hash.update(self.expose());
        hash.finalize().into()
    }
}
impl fmt::Debug for ServiceToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServiceToken([REDACTED])")
    }
}

/// Only confirmed issuance constructs these metadata. Not an authorization receipt.
#[derive(Debug)]
pub struct IssuedServiceToken {
    token: ServiceToken,
    scope: ServiceScope,
    created: u64,
    expires: u64,
    last_observed: u64,
    blocked: bool,
}
impl IssuedServiceToken {
    pub fn token(&self) -> &ServiceToken {
        &self.token
    }
    pub fn scope(&self) -> ServiceScope {
        self.scope
    }
    pub fn created_at(&self) -> u64 {
        self.created
    }
    pub fn expires_at(&self) -> u64 {
        self.expires
    }
    fn remaining_at(&mut self, time: u64) -> Result<u16, ServiceTokenError> {
        if self.blocked || time < self.created || time < self.last_observed || time >= self.expires
        {
            self.blocked = true;
            return Err(ServiceTokenError);
        }
        self.last_observed = time;
        let remaining = self.expires - time;
        if !(1..=300).contains(&remaining) {
            self.blocked = true;
            return Err(ServiceTokenError);
        }
        u16::try_from(remaining).map_err(|_| ServiceTokenError)
    }
}

/// Fresh database-time snapshot for response generation, not a current grant/peer check.
pub async fn remaining_lifetime(
    c: &mut PgConnection,
    issued: &mut IssuedServiceToken,
) -> Result<u16, ServiceTokenError> {
    autocommit(c)?;
    if issued.blocked {
        return Err(ServiceTokenError);
    }
    let time = match now(c).await {
        Ok(time) => time,
        Err(error) => {
            issued.blocked = true;
            return Err(error);
        }
    };
    issued.remaining_at(time)
}
#[derive(Debug)]
pub struct ActiveServiceToken {
    issuer: String,
    client: ServiceName,
    target: ServiceName,
    scope: ServiceScope,
    certificate: [u8; 32],
    created: u64,
    expires: u64,
    observed: u64,
}
impl ActiveServiceToken {
    pub(crate) fn observed_at(&self) -> u64 {
        self.observed
    }
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn client(&self) -> ServiceName {
        self.client
    }
    pub fn target(&self) -> ServiceName {
        self.target
    }
    pub fn scope(&self) -> ServiceScope {
        self.scope
    }
    pub fn certificate(&self) -> [u8; 32] {
        self.certificate
    }
    pub fn created_at(&self) -> u64 {
        self.created
    }
    pub fn expires_at(&self) -> u64 {
        self.expires
    }
}
fn autocommit(c: &PgConnection) -> Result<(), ServiceTokenError> {
    if c.is_in_transaction() {
        Err(ServiceTokenError)
    } else {
        Ok(())
    }
}
async fn now(c: &mut PgConnection) -> Result<u64, ServiceTokenError> {
    sqlx::query_scalar::<_, String>("SELECT floor(EXTRACT(epoch FROM clock_timestamp()))::text")
        .fetch_one(c)
        .await?
        .parse()
        .map_err(|_| ServiceTokenError)
}
fn number(row: &sqlx::postgres::PgRow, name: &str) -> Result<u64, ServiceTokenError> {
    row.try_get::<String, _>(name)?
        .parse()
        .map_err(|_| ServiceTokenError)
}
async fn authority(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
) -> Result<(), ServiceTokenError> {
    let matches:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM identity_access.service_authority WHERE singleton AND installation=$1 AND issuer=$2)")
        .bind(config.installation.as_slice()).bind(&config.issuer).fetch_one(c).await?;
    if matches {
        Ok(())
    } else {
        Err(ServiceTokenError)
    }
}
async fn peer(
    c: &mut PgConnection,
    reference: PeerReference,
    time: u64,
) -> Result<u64, ServiceTokenError> {
    let row=sqlx::query("SELECT trim_scale(cert.not_before)::text AS not_before,trim_scale(cert.not_after)::text AS not_after FROM identity_access.service_certificates cert JOIN identity_access.service_clients client ON client.name=cert.client WHERE client.name=$1 AND client.active AND cert.fingerprint=$2 AND NOT cert.revoked")
        .bind(reference.service.text()).bind(reference.certificate.as_slice()).fetch_one(&mut *c).await?;
    let after = now(c).await?;
    if after >= time && time >= number(&row, "not_before")? && after < number(&row, "not_after")? {
        Ok(number(&row, "not_after")?)
    } else {
        Err(ServiceTokenError)
    }
}
async fn save(
    c: &mut PgConnection,
    token: &ServiceToken,
    time: u64,
    state: &str,
) -> Result<(), ServiceTokenError> {
    let written=sqlx::query("UPDATE identity_access.service_tokens SET last_observed=$2::text::numeric,state=$3 WHERE fingerprint=$1")
        .bind(token.fingerprint().as_slice()).bind(time.to_string()).bind(state).execute(c).await?;
    if written.rows_affected() == 1 {
        Ok(())
    } else {
        Err(ServiceTokenError)
    }
}
/// Trusted internal issuance only. No certificate validation or HTTP endpoint implied.
pub async fn issue(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
    caller: PeerReference,
    target: ServiceName,
    scope: ServiceScope,
    lifetime_seconds: u16,
) -> Result<ServiceToken, ServiceTokenError> {
    issue_with_metadata(c, config, caller, target, scope, lifetime_seconds)
        .await
        .map(|issued| issued.token)
}
/// Confirmed internal issuance with actual immutable deadline metadata; still no HTTP endpoint.
pub async fn issue_with_metadata(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
    caller: PeerReference,
    target: ServiceName,
    scope: ServiceScope,
    lifetime_seconds: u16,
) -> Result<IssuedServiceToken, ServiceTokenError> {
    autocommit(c)?;
    if !(1..=300).contains(&lifetime_seconds) {
        return Err(ServiceTokenError);
    }
    let mut tx = c.begin_with("BEGIN ISOLATION LEVEL READ COMMITTED").await?;
    let result:Result<IssuedServiceToken,ServiceTokenError>=async{
        authority(&mut tx,config).await?;
        let row=sqlx::query("SELECT trim_scale(c.version)::text AS client_version,trim_scale(t.version)::text AS target_version,trim_scale(g.version)::text AS grant_version,trim_scale(cert.not_before)::text AS not_before,trim_scale(cert.not_after)::text AS not_after FROM identity_access.service_clients c JOIN identity_access.service_clients t ON t.name=$2 JOIN identity_access.service_grants g ON g.client=c.name AND g.target=t.name AND g.scope=$3 JOIN identity_access.service_certificates cert ON cert.client=c.name AND cert.fingerprint=$4 WHERE c.name=$1 AND c.active AND t.active AND g.active AND NOT cert.revoked")
            .bind(caller.service.text()).bind(target.text()).bind(scope.text()).bind(caller.certificate.as_slice()).fetch_one(&mut *tx).await?;
        let time=now(&mut tx).await?;
        let cert_end=number(&row,"not_after")?;
        if time<number(&row,"not_before")? || time>=cert_end{return Err(ServiceTokenError);}
        let expires=time.checked_add(u64::from(lifetime_seconds)).ok_or(ServiceTokenError)?.min(cert_end);
        let token=ServiceToken::generate()?;
        let written=sqlx::query("INSERT INTO identity_access.service_tokens (fingerprint,installation,issuer,client,target,scope,certificate,client_version,target_version,grant_version,created_at,expires_at,last_observed,state) VALUES($1,$2,$3,$4,$5,$6,$7,$8::text::numeric,$9::text::numeric,$10::text::numeric,$11::text::numeric,$12::text::numeric,$11::text::numeric,'active')")
            .bind(token.fingerprint().as_slice()).bind(config.installation.as_slice()).bind(&config.issuer).bind(caller.service.text()).bind(target.text()).bind(scope.text()).bind(caller.certificate.as_slice())
            .bind(number(&row,"client_version")?.to_string()).bind(number(&row,"target_version")?.to_string()).bind(number(&row,"grant_version")?.to_string()).bind(time.to_string()).bind(expires.to_string()).execute(&mut *tx).await?;
        if written.rows_affected()!=1{return Err(ServiceTokenError);}
        // An INSERT may wait behind another writer; never return an already expired bearer.
        let final_time=now(&mut tx).await?;
        if final_time<time || final_time>=expires{return Err(ServiceTokenError);}
        peer(&mut tx,caller,final_time).await?;
        let after_peer=now(&mut tx).await?;
        if after_peer<final_time || after_peer>=expires{return Err(ServiceTokenError);}
        Ok(IssuedServiceToken { token, scope, created: time, expires, last_observed: after_peer, blocked: false })
    }.await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}
/// Current internal metadata only. The later receiver must compare its actual TLS peer.
pub async fn introspect(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
    resource: PeerReference,
    token: &ServiceToken,
) -> Result<Option<ActiveServiceToken>, ServiceTokenError> {
    autocommit(c)?;
    let mut tx = c.begin_with("BEGIN ISOLATION LEVEL READ COMMITTED").await?;
    let result = introspect_in_transaction(&mut tx, config, resource, token).await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}
async fn introspect_in_transaction(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
    resource: PeerReference,
    token: &ServiceToken,
) -> Result<Option<ActiveServiceToken>, ServiceTokenError> {
    authority(&mut *c, config).await?;
    let initial_time = now(&mut *c).await?;
    peer(&mut *c, resource, initial_time).await?;
    let row=sqlx::query("SELECT installation,issuer,client,target,scope,certificate,trim_scale(client_version)::text AS client_version,trim_scale(target_version)::text AS target_version,trim_scale(grant_version)::text AS grant_version,trim_scale(created_at)::text AS created_at,trim_scale(expires_at)::text AS expires_at,trim_scale(last_observed)::text AS last_observed,state FROM identity_access.service_tokens WHERE fingerprint=$1 FOR UPDATE")
        .bind(token.fingerprint().as_slice()).fetch_optional(&mut *c).await?;
    let Some(row) = row else {
        return Ok(None);
    };
    // A wrong resource may not revoke somebody else's otherwise valid token.
    if row.try_get::<&str, _>("target")? != resource.service.text() {
        return Ok(None);
    }
    let time = now(&mut *c).await?;
    let created = number(&row, "created_at")?;
    let expires = number(&row, "expires_at")?;
    if time < number(&row, "last_observed")? || time < created {
        return Err(ServiceTokenError);
    }
    if row.try_get::<&str, _>("state")? != "active" {
        return Ok(None);
    }
    if time >= expires {
        save(&mut *c, token, time, "expired").await?;
        return Ok(None);
    }
    if row.try_get::<Vec<u8>, _>("installation")? != config.installation
        || row.try_get::<String, _>("issuer")? != config.issuer
    {
        return Err(ServiceTokenError);
    }
    let client = ServiceName::parse(row.try_get("client")?)?;
    let scope = ServiceScope::parse(row.try_get("scope")?)?;
    let certificate: [u8; 32] = row
        .try_get::<Vec<u8>, _>("certificate")?
        .try_into()
        .map_err(|_| ServiceTokenError)?;
    let current:bool=sqlx::query_scalar("SELECT EXISTS(SELECT FROM identity_access.service_clients client JOIN identity_access.service_clients target ON target.name=$2 JOIN identity_access.service_grants g ON (g.client,g.target,g.scope)=(client.name,target.name,$3) JOIN identity_access.service_certificates cert ON cert.client=client.name AND cert.fingerprint=$4 WHERE client.name=$1 AND client.active AND target.active AND g.active AND NOT cert.revoked AND (client.version,target.version,g.version)=($5::text::numeric,$6::text::numeric,$7::text::numeric) AND cert.not_before<=$8::text::numeric AND cert.not_after>$8::text::numeric)")
        .bind(client.text()).bind(resource.service.text()).bind(scope.text()).bind(certificate.as_slice()).bind(number(&row,"client_version")?.to_string()).bind(number(&row,"target_version")?.to_string()).bind(number(&row,"grant_version")?.to_string()).bind(time.to_string()).fetch_one(&mut *c).await?;
    let final_time = now(&mut *c).await?;
    if final_time < time {
        return Err(ServiceTokenError);
    }
    let resource_end = peer(&mut *c, resource, final_time).await?;
    let state = if final_time >= expires {
        "expired"
    } else if !current {
        "revoked"
    } else {
        "active"
    };
    save(&mut *c, token, final_time, state).await?;
    if state != "active" {
        return Ok(None);
    }
    let after_write = now(&mut *c).await?;
    if after_write < final_time {
        return Err(ServiceTokenError);
    }
    if after_write >= expires {
        save(&mut *c, token, after_write, "expired").await?;
        return Ok(None);
    }
    if after_write >= resource_end {
        return Err(ServiceTokenError);
    }
    Ok(Some(ActiveServiceToken {
        issuer: config.issuer.clone(),
        client,
        target: resource.service,
        scope,
        certificate,
        created,
        expires,
        observed: after_write,
    }))
}
/// Internal administrative revocation only. Authorization belongs to its trusted caller.
pub async fn revoke(c: &mut PgConnection, token: &ServiceToken) -> Result<bool, ServiceTokenError> {
    autocommit(c)?;
    let changed=sqlx::query("UPDATE identity_access.service_tokens SET state='revoked' WHERE fingerprint=$1 AND state='active'")
        .bind(token.fingerprint().as_slice()).execute(c).await?;
    Ok(changed.rows_affected() == 1)
}

#[cfg(test)]
mod output_tests {
    use super::*;
    fn issued() -> IssuedServiceToken {
        IssuedServiceToken {
            token: ServiceToken::from_bytes(&[9; 32]).unwrap(),
            scope: ServiceScope::MembershipRead,
            created: 100,
            expires: 400,
            last_observed: 100,
            blocked: false,
        }
    }
    // Deterministic metadata state; no real system or database clock is changed.
    #[test]
    fn output_deadline_rejects_regression_permanently() {
        let mut value = issued();
        assert_eq!(value.remaining_at(200).unwrap(), 200);
        assert!(value.remaining_at(199).is_err());
        assert!(value.remaining_at(201).is_err());
    }
    #[test]
    fn output_deadline_cannot_be_revived_after_observed_expiry() {
        let mut value = issued();
        assert!(value.remaining_at(400).is_err());
        assert!(value.remaining_at(200).is_err());
        assert!(
            !format!("{value:?}")
                .contains(crate::service_oauth::encode_token(value.token()).as_str())
        );
    }
}
