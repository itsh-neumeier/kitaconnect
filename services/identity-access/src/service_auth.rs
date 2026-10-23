//! Actual connection plus current registry. No token, user or operation authorization.
use crate::service_tokens::{
    AuthorityConfiguration, PeerReference, ServiceName, ServiceTokenError,
};
use kitaconnect_service_transport::{ConnectionPeer, RegisteredDnsName};
use sqlx::{Connection, PgConnection, Row};

/// Explicit static registration metadata. It does not cache active database state.
pub struct RegisteredNames(Vec<(ServiceName, RegisteredDnsName)>);
impl RegisteredNames {
    pub fn new(entries: Vec<(ServiceName, RegisteredDnsName)>) -> Result<Self, ServiceTokenError> {
        if entries.is_empty() {
            return Err(ServiceTokenError);
        }
        for (index, (service, name)) in entries.iter().enumerate() {
            if entries[..index]
                .iter()
                .any(|(other, dns)| other == service || dns == name)
            {
                return Err(ServiceTokenError);
            }
        }
        Ok(Self(entries))
    }
    fn expected(&self, service: ServiceName) -> Result<&RegisteredDnsName, ServiceTokenError> {
        self.0
            .iter()
            .find(|(registered, _)| *registered == service)
            .map(|(_, name)| name)
            .ok_or(ServiceTokenError)
    }
}

/// Fresh service authentication only. Call again for the next protected check, even on keep-alive.
pub async fn authenticate(
    c: &mut PgConnection,
    config: &AuthorityConfiguration,
    names: &RegisteredNames,
    claimed: ServiceName,
    connection: &ConnectionPeer,
) -> Result<PeerReference, ServiceTokenError> {
    if c.is_in_transaction() {
        return Err(ServiceTokenError);
    }
    let expected = names.expected(claimed)?;
    let actual = connection.certificate().map_err(|_| ServiceTokenError)?;
    if !actual.matches_dns_name(expected) {
        return Err(ServiceTokenError);
    }
    let digest = actual.fingerprint();
    let start = actual.not_before();
    let end = actual.not_after();
    let mut tx = c
        .begin_with("BEGIN ISOLATION LEVEL READ COMMITTED READ ONLY")
        .await?;
    let result: Result<PeerReference, ServiceTokenError> = async {
        let row = sqlx::query("SELECT trim_scale(cert.not_before)::text AS not_before,trim_scale(cert.not_after)::text AS not_after,floor(extract(epoch FROM clock_timestamp()))::text AS observed FROM identity_access.service_certificates cert JOIN identity_access.service_clients client ON client.name=cert.client JOIN identity_access.service_authority authority ON authority.singleton WHERE client.name=$1 AND client.active AND cert.fingerprint=$2 AND NOT cert.revoked AND authority.installation=$3 AND authority.issuer=$4")
            .bind(claimed.text()).bind(digest.as_slice()).bind(config.installation().as_slice()).bind(config.issuer())
            .fetch_one(&mut *tx).await?;
        let number = |name: &str| -> Result<u64, ServiceTokenError> {
            row.try_get::<String, _>(name)?.parse().map_err(|_| ServiceTokenError)
        };
        let now: u64 = sqlx::query_scalar::<_, String>("SELECT floor(extract(epoch FROM clock_timestamp()))::text")
            .fetch_one(&mut *tx).await?.parse().map_err(|_| ServiceTokenError)?;
        if number("not_before")? != start || number("not_after")? != end
            || now < number("observed")? || now < start || now >= end {
            return Err(ServiceTokenError);
        }
        connection.certificate().map_err(|_| ServiceTokenError)?;
        Ok(PeerReference::new(claimed, digest))
    }.await;
    match result {
        Ok(reference) => {
            tx.commit().await?;
            connection.certificate().map_err(|_| ServiceTokenError)?;
            Ok(reference)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}
