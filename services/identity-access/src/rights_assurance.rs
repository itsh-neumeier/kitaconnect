//! Current, complete scoped role classifications. Never an object/field permission.
use crate::{
    IdentityId, login::LoginFacilityId, operation_assurance::OperationClass,
    session_check::Requirement,
};
use serde::Deserialize;
use sqlx::{Connection, PgConnection, Row, postgres::PgConnectOptions};
use std::{
    collections::HashSet,
    fmt,
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, time::timeout};

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

#[derive(Clone, Copy)]
pub struct ReadLimits {
    pub parallel: usize,
    pub connect_time: Duration,
    pub database_time: Duration,
    pub total_time: Duration,
}
pub struct Reader {
    database: PgConnectOptions,
    limits: ReadLimits,
    budget: Semaphore,
}
fn database_timeout(duration: Duration) -> Result<String, ReadError> {
    let ms = duration.as_nanos().div_ceil(1_000_000);
    if ms == 0 || ms > i32::MAX as u128 {
        return Err(ReadError);
    }
    Ok(ms.to_string())
}
impl Reader {
    /// Own fresh connections only: never borrows a caller's transaction snapshot.
    pub fn new(database: PgConnectOptions, limits: ReadLimits) -> Result<Self, ReadError> {
        if limits.parallel == 0
            || limits.parallel > Semaphore::MAX_PERMITS
            || limits.connect_time > limits.total_time
            || [limits.connect_time, limits.database_time, limits.total_time]
                .iter()
                .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(ReadError);
        }
        let ms = database_timeout(limits.database_time)?;
        let database = database
            .application_name("kc_role_assurance_request")
            .options([
                ("statement_timeout", ms.as_str()),
                ("transaction_timeout", ms.as_str()),
                ("idle_in_transaction_session_timeout", ms.as_str()),
            ]);
        Ok(Self {
            database,
            limits,
            budget: Semaphore::new(limits.parallel),
        })
    }
    pub async fn read(
        &self,
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<CurrentAssurance, ReadError> {
        let _permit = self.budget.try_acquire().map_err(|_| ReadError)?;
        timeout(self.limits.total_time, async {
            let mut connection = timeout(
                self.limits.connect_time,
                PgConnection::connect_with(&self.database),
            )
            .await
            .map_err(|_| ReadError)?
            .map_err(|_| ReadError)?;
            read_current(&mut connection, identity, facility).await
        })
        .await
        .map_err(|_| ReadError)?
    }
}

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
            .is_some_and(|roles| {
                roles.iter().all(|role| {
                    role.is_object()
                        && role
                            .get("operations")
                            .and_then(|v| v.as_array())
                            .is_some_and(|operations| operations.iter().all(|v| v.is_string()))
                })
            })
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
/// Only Reader's owned fresh connection can reach this internal operation.
async fn read_current(
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
            r#"{"format":1,"roles":[{"key":"a","operations":[{"health_data":null}]}]}"#,
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

    #[test]
    fn connection_and_server_limits_are_explicit_and_representable() {
        let limits = ReadLimits {
            parallel: 1,
            connect_time: Duration::from_secs(1),
            database_time: Duration::from_secs(1),
            total_time: Duration::from_secs(2),
        };
        let valid = |limits| Reader::new(PgConnectOptions::new(), limits).is_ok();
        assert!(valid(limits));
        for parallel in [0, Semaphore::MAX_PERMITS + 1] {
            assert!(!valid(ReadLimits { parallel, ..limits }));
        }
        for value in [Duration::ZERO, Duration::MAX] {
            assert!(!valid(ReadLimits {
                connect_time: value,
                ..limits
            }));
            assert!(!valid(ReadLimits {
                database_time: value,
                ..limits
            }));
            assert!(!valid(ReadLimits {
                total_time: value,
                ..limits
            }));
        }
        assert!(!valid(ReadLimits {
            connect_time: Duration::from_secs(3),
            ..limits
        }));
        assert_eq!(database_timeout(Duration::from_nanos(1)).unwrap(), "1");
        assert_eq!(database_timeout(Duration::from_micros(1001)).unwrap(), "2");
        assert!(database_timeout(Duration::from_millis(i32::MAX as u64 + 1)).is_err());
    }
}
