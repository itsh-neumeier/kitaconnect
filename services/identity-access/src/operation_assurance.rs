//! Authentication requirements only. Trusted routes classify the actual action/data;
//! browser-supplied labels are not authority. Domain rights remain a separate check.
use crate::{
    login::{LoginFacilityId, OrganizationAuthority},
    session_check::{self, CheckError, CurrentSession, Requirement},
    token::SessionToken,
};
use sqlx::PgConnection;

#[derive(Clone, Copy, Debug)]
pub enum OperationClass {
    RoleAdministration,
    SecuritySettings,
    HealthData,
    CustodyRestrictions,
    PersonnelFile,
    InternalDevelopmentRecord,
    PublishedPortfolio,
    BasicContactData,
    Unknown,
}
impl OperationClass {
    pub(crate) fn requirement(self) -> Result<Requirement, CheckError> {
        match self {
            Self::RoleAdministration
            | Self::SecuritySettings
            | Self::HealthData
            | Self::CustodyRestrictions
            | Self::PersonnelFile
            | Self::InternalDevelopmentRecord => Ok(Requirement::RecentMultiFactor),
            Self::PublishedPortfolio | Self::BasicContactData => Ok(Requirement::Standard),
            Self::Unknown => Err(CheckError),
        }
    }
}

/// A current authentication snapshot, never permission to read or modify an object.
/// The trusted route must also enforce current object/field and organizational rights.
pub async fn check(
    c: &mut PgConnection,
    token: &SessionToken,
    facility: LoginFacilityId,
    operation: OperationClass,
    authority: &mut impl OrganizationAuthority,
) -> Result<CurrentSession, CheckError> {
    session_check::check(c, token, facility, operation.requirement()?, authority).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmed_sensitive_operations_cannot_select_standard_authentication() {
        for operation in [
            OperationClass::RoleAdministration,
            OperationClass::SecuritySettings,
            OperationClass::HealthData,
            OperationClass::CustodyRestrictions,
            OperationClass::PersonnelFile,
            OperationClass::InternalDevelopmentRecord,
        ] {
            assert!(matches!(
                operation.requirement(),
                Ok(Requirement::RecentMultiFactor)
            ));
        }
        for operation in [
            OperationClass::PublishedPortfolio,
            OperationClass::BasicContactData,
        ] {
            assert!(matches!(operation.requirement(), Ok(Requirement::Standard)));
        }
        assert!(OperationClass::Unknown.requirement().is_err());
    }
}
