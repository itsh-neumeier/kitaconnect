//! Complete actual source composition; no callback stands in for a current authority.
use super::*;
use kitaconnect_identity_access::{
    current_login_source::{
        Configuration as JoinedConfiguration, ReadLimits as JoinedLimits, Reader as JoinedReader,
    },
    rights_assurance::{ReadLimits as RightsLimits, Reader as RightsReader},
};
#[path = "login_chain.rs"]
pub(super) mod login_chain;

fn joined(
    f: &Fixture<'_, '_>,
    identity: &Leaf,
    issuer_address: SocketAddr,
    organization_address: SocketAddr,
    pin: [u8; 32],
    database: PgConnectOptions,
) -> JoinedReader {
    let issuer = OAuthHttpClient::new(
        Client::new(
            identity.chain.clone(),
            identity.key.clone_key(),
            vec![f.ca.der().clone()],
        )
        .unwrap(),
        RegisteredDnsName::new("identity.test").unwrap(),
        ClientLimits {
            parallel: 1,
            form_bytes: 512,
            response_bytes: 2048,
            headers: 16,
            header_bytes: 8192,
            handshake_time: Duration::from_secs(1),
            header_time: Duration::from_secs(1),
            body_time: Duration::from_secs(1),
            total_time: Duration::from_secs(2),
        },
    )
    .unwrap();
    JoinedReader::new(
        Arc::new(issuer),
        Arc::new(consumer(
            f,
            identity,
            organization_address,
            f.organization.digest,
            consumer_limits(),
        )),
        Arc::new(
            RightsReader::new(
                database,
                RightsLimits {
                    parallel: 1,
                    connect_time: Duration::from_secs(1),
                    database_time: Duration::from_millis(500),
                    total_time: Duration::from_secs(2),
                },
            )
            .unwrap(),
        ),
        JoinedConfiguration {
            address: issuer_address,
            server_certificate: pin,
            limits: JoinedLimits {
                parallel: 1,
                response_bytes: 2048,
                connect_time: Duration::from_secs(1),
                total_time: Duration::from_secs(5),
            },
        },
    )
    .unwrap()
}
async fn read(
    reader: &JoinedReader,
    site: FacilityId,
) -> Result<
    kitaconnect_identity_access::current_login_source::CurrentLogin,
    kitaconnect_identity_access::current_login_source::ReadError,
> {
    reader
        .read(
            IdentityId::new(1).unwrap(),
            LoginFacilityId::new(site.value()).unwrap(),
        )
        .await
}
pub(super) async fn check(
    f: &Fixture<'_, '_>,
    identity: &Leaf,
    issuer_address: SocketAddr,
    organization_address: SocketAddr,
    site: FacilityId,
    other: FacilityId,
) -> (JoinedReader, login_chain::LiveSession) {
    let mut admin = PgConnection::connect_with(&options("postgres"))
        .await
        .unwrap();
    if site.value() == 101 {
        let persisted: String = sqlx::query_scalar("SELECT version::text FROM identity_access.role_assurance_scopes WHERE identity_id=1 AND facility_id=1 AND state='revoked'")
            .fetch_one(&mut admin).await.unwrap();
        assert_eq!(persisted, "5");
    }
    sqlx::query("INSERT INTO identity_access.identities VALUES(1) ON CONFLICT DO NOTHING")
        .execute(&mut admin)
        .await
        .unwrap();
    let basic = r#"{"format":1,"roles":[{"key":"synthetic","operations":["basic_contact_data"]}]}"#;
    let sensitive = r#"{"format":1,"roles":[{"key":"synthetic","operations":["health_data"]}]}"#;
    sqlx::query("INSERT INTO identity_access.role_assurance_scopes VALUES(1,$1::text::numeric,1,'active',$2)")
        .bind(site.value().to_string()).bind(basic).execute(&mut admin).await.unwrap();
    let mut reader = joined(
        f,
        identity,
        issuer_address,
        organization_address,
        identity.digest,
        options("postgres"),
    );
    let current = read(&reader, site).await.unwrap();
    assert_eq!(
        (current.identity().value(), current.facility().value()),
        (1, site.value())
    );
    assert_eq!(
        (
            current.membership_version(),
            current.policy_version(),
            current.rights_version()
        ),
        (1, 1, 1)
    );
    assert_eq!(
        (
            current.idle_seconds(),
            current.absolute_seconds(),
            current.step_up_seconds()
        ),
        (900, 28800, 300)
    );
    assert!(!current.mfa_required());
    {
        use kitaconnect_identity_access::login::OrganizationAuthority;
        let context = reader
            .current(
                IdentityId::new(1).unwrap(),
                LoginFacilityId::new(site.value()).unwrap(),
            )
            .await
            .unwrap();
        let versions = context
            .source_versions()
            .expect("actual port must preserve all sources");
        assert_eq!(
            (versions.membership(), versions.policy(), versions.rights()),
            (1, 1, 1)
        );
    }
    assert_eq!(format!("{current:?}"), "CurrentLogin(REDACTED)");
    assert!(read(&reader, other).await.is_err());
    assert!(
        reader
            .read(
                IdentityId::new(2).unwrap(),
                LoginFacilityId::new(site.value()).unwrap()
            )
            .await
            .is_err()
    );

    // Pin mismatch must fail even though the issuer has a valid CA/DNS certificate.
    assert!(
        read(
            &joined(
                f,
                identity,
                issuer_address,
                organization_address,
                [99; 32],
                options("postgres")
            ),
            site
        )
        .await
        .is_err()
    );
    // An active membership cannot replace an invisible or unavailable own role source.
    assert!(
        read(
            &joined(
                f,
                identity,
                issuer_address,
                organization_address,
                identity.digest,
                options("kc_service_http_runtime")
            ),
            site
        )
        .await
        .is_err()
    );
    assert!(
        read(
            &joined(
                f,
                identity,
                issuer_address,
                organization_address,
                identity.digest,
                options("postgres").database("kc_missing_synthetic_database")
            ),
            site
        )
        .await
        .is_err()
    );
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=2 WHERE identity_id=1 AND facility_id=$2::text::numeric")
        .bind(sensitive).bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    let current = read(&reader, site).await.unwrap();
    assert_eq!(
        (
            current.membership_version(),
            current.policy_version(),
            current.rights_version()
        ),
        (1, 1, 2)
    );
    assert!(current.mfa_required());

    let mut organization = PgConnection::connect_with(&org_options("postgres"))
        .await
        .unwrap();
    sqlx::query("UPDATE organization_config.facility_session_policies SET version=2,idle_seconds=300,absolute_seconds=7200,step_up_seconds=120 WHERE facility_id=$1::text::numeric")
        .bind(site.value().to_string()).execute(&mut organization).await.unwrap();
    let current = read(&reader, site).await.unwrap();
    assert_eq!(
        (
            current.membership_version(),
            current.policy_version(),
            current.rights_version()
        ),
        (1, 2, 2)
    );
    assert_eq!(
        (
            current.idle_seconds(),
            current.absolute_seconds(),
            current.step_up_seconds()
        ),
        (300, 7200, 120)
    );

    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=3 WHERE identity_id=1 AND facility_id=$2::text::numeric")
        .bind(r#"{"format":1,"roles":[{"key":"synthetic","operations":["unknown"]}]}"#).bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    assert!(read(&reader, site).await.is_err());
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET document=$1,version=4 WHERE identity_id=1 AND facility_id=$2::text::numeric")
        .bind(sensitive).bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    assert_eq!(read(&reader, site).await.unwrap().rights_version(), 4);
    sqlx::query("UPDATE identity_access.role_assurance_scopes SET state='revoked',version=5 WHERE identity_id=1 AND facility_id=$1::text::numeric")
        .bind(site.value().to_string()).execute(&mut admin).await.unwrap();
    assert!(read(&reader, site).await.is_err());
    let live = login_chain::check(f, identity, issuer_address, organization_address, site).await;
    (reader, live)
}
