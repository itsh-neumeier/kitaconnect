//! Privileged maintenance only: empty facility schemas and their registry entry.
//! No client schema names, runtime role switch, application grants or business tables.

use crate::FacilityId;
use sqlx::{Connection, PgConnection};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredSchema(String);

impl RegisteredSchema {
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum ProvisionError {
    MaintenanceRequired,
    UnsafeRuntimeRole,
    UnknownFacility,
    AlreadyRegistered,
    SchemaExists,
    InvalidRegistration,
    Database(sqlx::Error),
}

impl fmt::Display for ProvisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MaintenanceRequired => "facility maintenance identity required",
            Self::UnsafeRuntimeRole => "unsafe runtime role configuration",
            Self::UnknownFacility => "unknown facility reference",
            Self::AlreadyRegistered => "facility schema already registered",
            Self::SchemaExists => "facility schema already exists",
            Self::InvalidRegistration => "invalid facility schema registration",
            Self::Database(_) => "facility provisioning failed",
        })
    }
}

impl std::error::Error for ProvisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ProvisionError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

fn canonical_name(facility: FacilityId) -> String {
    format!("facility_{:032x}", facility.value())
}

async fn preflight(connection: &mut PgConnection) -> Result<(), ProvisionError> {
    let maintenance: bool = sqlx::query_scalar(
        "SELECT pg_catalog.pg_has_role(current_user, 'kc_facility_provisioner', 'USAGE')",
    )
    .fetch_one(&mut *connection)
    .await?;
    if !maintenance {
        return Err(ProvisionError::MaintenanceRequired);
    }
    let safe: Option<bool> = sqlx::query_scalar(
        "SELECT NOT (r.rolcanlogin OR r.rolsuper OR r.rolcreatedb OR r.rolcreaterole OR r.rolreplication OR r.rolbypassrls) \
         AND NOT pg_catalog.has_database_privilege(r.oid, pg_catalog.current_database(), 'CREATE') \
         AND NOT EXISTS (SELECT 1 FROM pg_catalog.pg_auth_members m WHERE m.member = r.oid) \
         AND NOT EXISTS (SELECT 1 FROM pg_catalog.pg_shdepend d WHERE d.refclassid = 'pg_catalog.pg_authid'::pg_catalog.regclass \
                         AND d.refobjid = r.oid AND d.deptype = 'o') \
         FROM pg_catalog.pg_roles r WHERE r.rolname = 'kc_org_runtime'",
    )
    .fetch_optional(connection)
    .await?;
    if safe != Some(true) {
        return Err(ProvisionError::UnsafeRuntimeRole);
    }
    Ok(())
}

async fn physical_schema_exists(
    connection: &mut PgConnection,
    name: &str,
) -> Result<bool, ProvisionError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = $1)",
    )
    .bind(name)
    .fetch_one(connection)
    .await?)
}

/// Returns a checked maintenance registry value, never a user access capability.
pub async fn resolve(
    connection: &mut PgConnection,
    facility: FacilityId,
) -> Result<Option<RegisteredSchema>, ProvisionError> {
    preflight(connection).await?;
    let name: Option<String> = sqlx::query_scalar(
        "SELECT schema_name FROM organization_config.facility_schemas WHERE facility_id = $1::text::numeric",
    )
    .bind(facility.value().to_string())
    .fetch_optional(&mut *connection)
    .await?;
    let Some(name) = name else { return Ok(None) };
    if name != canonical_name(facility) {
        return Err(ProvisionError::InvalidRegistration);
    }
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace n \
         WHERE n.nspname = $1 AND n.nspowner = 'kc_facility_schema_owner'::pg_catalog.regrole \
         AND NOT pg_catalog.has_schema_privilege('kc_org_runtime', n.oid, 'USAGE') \
         AND NOT pg_catalog.has_schema_privilege('kc_org_runtime', n.oid, 'CREATE') \
         AND NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(COALESCE(n.nspacl, pg_catalog.acldefault('n', n.nspowner))) a \
                         WHERE a.grantee = 0 AND a.privilege_type IN ('USAGE', 'CREATE')))",
    )
    .bind(&name)
    .fetch_one(connection)
    .await?;
    if !valid {
        return Err(ProvisionError::InvalidRegistration);
    }
    Ok(Some(RegisteredSchema(name)))
}

async fn provision_in_transaction(
    connection: &mut PgConnection,
    facility: FacilityId,
) -> Result<RegisteredSchema, ProvisionError> {
    preflight(connection).await?;
    sqlx::query("SELECT pg_catalog.pg_advisory_xact_lock(184005, 1)")
        .execute(&mut *connection)
        .await?;
    let known: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM organization_config.facilities WHERE id = $1::text::numeric)",
    )
    .bind(facility.value().to_string())
    .fetch_one(&mut *connection)
    .await?;
    if !known {
        return Err(ProvisionError::UnknownFacility);
    }
    if resolve(connection, facility).await?.is_some() {
        return Err(ProvisionError::AlreadyRegistered);
    }
    let name = canonical_name(facility);
    if physical_schema_exists(connection, &name).await? {
        return Err(ProvisionError::SchemaExists);
    }
    // Only this fixed prefix and formatted u128 enter DDL; no external string is accepted.
    let create = sqlx::AssertSqlSafe(format!(
        "CREATE SCHEMA {name} AUTHORIZATION kc_facility_schema_owner"
    ));
    sqlx::query(create).execute(&mut *connection).await?;
    sqlx::query(
        "INSERT INTO organization_config.facility_schemas (facility_id, schema_name) VALUES ($1::text::numeric, $2)",
    )
    .bind(facility.value().to_string())
    .bind(&name)
    .execute(&mut *connection)
    .await?;
    resolve(connection, facility)
        .await?
        .ok_or(ProvisionError::InvalidRegistration)
}

/// Atomically creates an empty schema and registry entry under the supplied maintenance identity.
pub async fn provision(
    connection: &mut PgConnection,
    facility: FacilityId,
) -> Result<RegisteredSchema, ProvisionError> {
    let mut transaction = connection.begin().await?;
    match provision_in_transaction(&mut transaction, facility).await {
        Ok(registration) => {
            transaction.commit().await?;
            Ok(registration)
        }
        Err(error) => {
            transaction.rollback().await?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_names_preserve_the_full_identifier_without_sql_input() {
        assert_eq!(
            canonical_name(FacilityId::new(1).unwrap()),
            "facility_00000000000000000000000000000001"
        );
        assert_eq!(
            canonical_name(FacilityId::new(u128::MAX).unwrap()),
            "facility_ffffffffffffffffffffffffffffffff"
        );
        for value in [1, 2, 255, u128::MAX] {
            let name = canonical_name(FacilityId::new(value).unwrap());
            assert_eq!(name.len(), 41);
            assert!(
                name[9..]
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
        }
    }
}
