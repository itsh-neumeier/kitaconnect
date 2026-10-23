//! Internal storage operations. The caller supplies a connection with verified transport,
//! identity and current permissions. No migrations, role changes or permission grants.

use crate::{CarrierId, Facility, FacilityId, ModelError};
use sqlx::{PgConnection, Row};
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Model(ModelError),
    InvalidStoredIdentifier,
    WriteNotConfirmed,
    Database(sqlx::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(_) => f.write_str("organization model constraint failed"),
            Self::InvalidStoredIdentifier => f.write_str("invalid stored organization identifier"),
            Self::WriteNotConfirmed => f.write_str("organization write was not confirmed"),
            Self::Database(_) => f.write_str("organization storage operation failed"),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for StorageError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

fn constraint(error: sqlx::Error) -> StorageError {
    if let Some(db) = error.as_database_error() {
        let model_error = match (db.code().as_deref(), db.constraint()) {
            (Some("23505"), Some("carriers_pkey")) => Some(ModelError::DuplicateCarrier),
            (Some("23505"), Some("facilities_pkey")) => Some(ModelError::DuplicateFacility),
            (Some("23503"), Some("facilities_carrier_id_fkey")) => Some(ModelError::UnknownCarrier),
            _ => None,
        };
        if let Some(model_error) = model_error {
            return StorageError::Model(model_error);
        }
    }
    StorageError::Database(error)
}

fn stored_id(value: &str) -> Result<u128, StorageError> {
    value
        .parse::<u128>()
        .ok()
        .filter(|value| *value != 0)
        .ok_or(StorageError::InvalidStoredIdentifier)
}

pub async fn register_carrier(
    connection: &mut PgConnection,
    id: CarrierId,
) -> Result<(), StorageError> {
    let result =
        sqlx::query("INSERT INTO organization_config.carriers (id) VALUES ($1::text::numeric)")
            .bind(id.value().to_string())
            .execute(connection)
            .await
            .map_err(constraint)?;
    if result.rows_affected() != 1 {
        return Err(StorageError::WriteNotConfirmed);
    }
    Ok(())
}

pub async fn register_facility(
    connection: &mut PgConnection,
    id: FacilityId,
    carrier: Option<CarrierId>,
) -> Result<(), StorageError> {
    let result = sqlx::query(
        "INSERT INTO organization_config.facilities (id, carrier_id) VALUES ($1::text::numeric, $2::text::numeric)",
    )
    .bind(id.value().to_string())
    .bind(carrier.map(|id| id.value().to_string()))
    .execute(connection)
    .await
    .map_err(constraint)?;
    if result.rows_affected() != 1 {
        return Err(StorageError::WriteNotConfirmed);
    }
    Ok(())
}

/// Returns existence only within the connection's authorized visible scope.
pub async fn contains_carrier(
    connection: &mut PgConnection,
    id: CarrierId,
) -> Result<bool, StorageError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM organization_config.carriers WHERE id = $1::text::numeric)",
    )
    .bind(id.value().to_string())
    .fetch_one(connection)
    .await?)
}

/// No visible row is not proof of global nonexistence.
pub async fn facility(
    connection: &mut PgConnection,
    id: FacilityId,
) -> Result<Option<Facility>, StorageError> {
    let row = sqlx::query(
        "SELECT pg_catalog.trim_scale(id)::text AS id, pg_catalog.trim_scale(carrier_id)::text AS carrier_id \
         FROM organization_config.facilities WHERE id = $1::text::numeric",
    )
    .bind(id.value().to_string())
    .fetch_optional(connection)
    .await?;
    row.map(|row| {
        let raw_id: String = row.try_get("id")?;
        let raw_carrier: Option<String> = row.try_get("carrier_id")?;
        let id =
            FacilityId::new(stored_id(&raw_id)?).ok_or(StorageError::InvalidStoredIdentifier)?;
        let carrier = raw_carrier
            .map(|raw| {
                CarrierId::new(stored_id(&raw)?).ok_or(StorageError::InvalidStoredIdentifier)
            })
            .transpose()?;
        Ok(Facility { id, carrier })
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_stored_values_never_become_valid_identifiers() {
        for value in [
            "0",
            "-1",
            "1.5",
            "NaN",
            "Infinity",
            "340282366920938463463374607431768211456",
        ] {
            assert!(matches!(
                stored_id(value),
                Err(StorageError::InvalidStoredIdentifier)
            ));
        }
        assert_eq!(stored_id("1").unwrap(), 1);
        assert_eq!(stored_id(&u128::MAX.to_string()).unwrap(), u128::MAX);
    }
}
