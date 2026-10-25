//! Current, complete scoped role classifications. Never an object/field permission.
use crate::{
    IdentityId, login::LoginFacilityId, operation_assurance::OperationClass,
    session_check::Requirement,
};
use serde::Deserialize;
use sqlx::{Connection, PgConnection, Row};
use std::{collections::HashSet, fmt};

const DOCUMENT_BYTES: usize = 65_536;

#[derive(Clone, Copy)]
pub struct ReadError;
impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("current role assurance unavailable")
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for ReadError {}

pub struct CurrentAssurance {
    identity: IdentityId,
    facility: LoginFacilityId,
    version: u64,
    mfa_required: bool,
}
impl fmt::Debug for CurrentAssurance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CurrentAssurance(REDACTED)")
    }
}
impl CurrentAssurance {
    pub fn identity(&self) -> IdentityId {
        self.identity
    }
    pub fn facility(&self) -> LoginFacilityId {
        self.facility
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn mfa_required(&self) -> bool {
        self.mfa_required
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format: u8,
    roles: Vec<Role>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Role {
    key: String,
    operations: Vec<Class>,
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum Class {
    RoleAdministration,
    SecuritySettings,
    HealthData,
    CustodyRestrictions,
    PersonnelFile,
    InternalDevelopmentRecord,
    PublishedPortfolio,
    BasicContactData,
}
impl Class {
    fn operation(self) -> OperationClass {
        match self {
            Self::RoleAdministration => OperationClass::RoleAdministration,
            Self::SecuritySettings => OperationClass::SecuritySettings,
            Self::HealthData => OperationClass::HealthData,
            Self::CustodyRestrictions => OperationClass::CustodyRestrictions,
            Self::PersonnelFile => OperationClass::PersonnelFile,
            Self::InternalDevelopmentRecord => OperationClass::InternalDevelopmentRecord,
            Self::PublishedPortfolio => OperationClass::PublishedPortfolio,
            Self::BasicContactData => OperationClass::BasicContactData,
        }
    }
}

fn derive(document: &str) -> Result<bool, ReadError> {
    if document.len() > DOCUMENT_BYTES {
        return Err(ReadError);
    }
    // Serde's struct sequence support is not part of this object-only format.
    // This shape check does not replace the duplicate-field checks below.
    let shape: serde_json::Value = serde_json::from_str(document).map_err(|_| ReadError)?;
    if !shape.is_object()
        || !shape
            .get("roles")
            .and_then(|v| v.as_array())
            .is_some_and(|roles| roles.iter().all(|r| r.is_object()))
    {
        return Err(ReadError);
    }
    let value: Document = serde_json::from_str(document).map_err(|_| ReadError)?;
    if value.format != 1 || value.roles.len() > 64 {
        return Err(ReadError);
    }
    let mut keys = HashSet::new();
    let mut mfa = false;
    for role in value.roles {
        if role.key.is_empty()
            || role.key.len() > 64
            || !role
                .key
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
            || !keys.insert(role.key)
            || role.operations.is_empty()
            || role.operations.len() > 8
        {
            return Err(ReadError);
        }
        let mut operations = HashSet::new();
        for class in role.operations {
            if !operations.insert(class) {
                return Err(ReadError);
            }
            mfa |= matches!(
                class.operation().requirement().map_err(|_| ReadError)?,
                Requirement::RecentMultiFactor
            );
        }
    }
    Ok(mfa)
}

/// One current statement snapshot. No default on absent, hidden or revoked scope.
/// The caller supplies its already bounded connection; no connection/role is created.
pub async fn read_current(
    connection: &mut PgConnection,
    identity: IdentityId,
    facility: LoginFacilityId,
) -> Result<CurrentAssurance, ReadError> {
    if connection.is_in_transaction() {
        return Err(ReadError);
    }
    let row = sqlx::query("SELECT version::text AS version, document FROM identity_access.role_assurance_scopes WHERE identity_id=$1::text::numeric AND facility_id=$2::text::numeric AND state='active'")
        .bind(identity.value().to_string()).bind(facility.value().to_string())
        .fetch_optional(connection).await.map_err(|_| ReadError)?.ok_or(ReadError)?;
    let version: u64 = row
        .try_get::<String, _>("version")
        .map_err(|_| ReadError)?
        .parse()
        .map_err(|_| ReadError)?;
    if version == 0 {
        return Err(ReadError);
    }
    let document: String = row.try_get("document").map_err(|_| ReadError)?;
    Ok(CurrentAssurance {
        identity,
        facility,
        version,
        mfa_required: derive(&document)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn all_confirmed_classes_and_mixed_roles_use_shared_requirement() {
        for (class, expected) in [
            ("role_administration", true),
            ("security_settings", true),
            ("health_data", true),
            ("custody_restrictions", true),
            ("personnel_file", true),
            ("internal_development_record", true),
            ("published_portfolio", false),
            ("basic_contact_data", false),
        ] {
            let document = json!({"format":1,"roles":[{"key":"synthetic","operations":[class]}]});
            assert_eq!(derive(&document.to_string()).unwrap(), expected);
        }
        assert!(!derive(r#"{"format":1,"roles":[]}"#).unwrap());
        assert!(derive(r#"{"format":1,"roles":[{"key":"a","operations":["basic_contact_data"]},{"key":"b","operations":["health_data"]}]}"#).unwrap());
    }

    #[test]
    fn malformed_unknown_duplicate_and_sequence_documents_fail_closed() {
        for input in [
            "null",
            "[]",
            "{}",
            r#"[1,[]]"#,
            r#"{"format":1,"roles":[["a",["health_data"]]]}"#,
            r#"{"format":2,"roles":[]}"#,
            r#"{"format":1.0,"roles":[]}"#,
            r#"{"format":1,"roles":[],"mfa_required":false}"#,
            r#"{"format":1,"roles":[],"roles":[]}"#,
            r#"{"format":1,"roles":[],"\u0072oles":[]}"#,
            r#"{"format":1,"roles":[{"key":"a","operations":[]}] }"#,
            r#"{"format":1,"roles":[{"key":"a","operations":["unknown"]}]}"#,
            r#"{"format":1,"roles":[{"key":"a","operations":["basic_contact_data","basic_contact_data"]}]}"#,
            r#"{"format":1,"roles":[{"key":"a","key":"b","operations":["health_data"]}]}"#,
            r#"{"format":1,"roles":[{"key":"a","operations":["health_data"],"scope":"carrier"}]}"#,
            r#"{"format":1,"roles":[{"key":"a","operations":["health_data"]},{"key":"a","operations":["basic_contact_data"]}]}"#,
        ] {
            assert!(derive(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn explicit_storage_work_bounds_and_redaction() {
        let role = |key: String| json!({"key":key,"operations":["basic_contact_data"]});
        for key in [
            "".to_owned(),
            "A".to_owned(),
            "ä".to_owned(),
            "x".repeat(65),
        ] {
            assert!(derive(&json!({"format":1,"roles":[role(key)]}).to_string()).is_err());
        }
        let roles: Vec<_> = (0..64).map(|n| role(format!("role_{n}"))).collect();
        let mut document = json!({"format":1,"roles":roles});
        assert!(!derive(&document.to_string()).unwrap());
        document["roles"]
            .as_array_mut()
            .unwrap()
            .push(role("extra".to_owned()));
        assert!(derive(&document.to_string()).is_err());
        let base = r#"{"format":1,"roles":[]}"#;
        let bounded = format!("{base}{}", " ".repeat(DOCUMENT_BYTES - base.len()));
        assert!(!derive(&bounded).unwrap());
        assert!(derive(&(bounded + " ")).is_err());
        assert_eq!(
            format!("{ReadError:?}"),
            "current role assurance unavailable"
        );
        let value = CurrentAssurance {
            identity: IdentityId::new(1).unwrap(),
            facility: LoginFacilityId::new(2).unwrap(),
            version: 1,
            mfa_required: true,
        };
        assert_eq!(format!("{value:?}"), "CurrentAssurance(REDACTED)");
    }
}
