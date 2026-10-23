//! Internal organization model, persistence and current service-authenticated membership data.
//! No public router or user/MFA/object authorization is provided by this library.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU128;

pub mod authenticated_membership;
pub mod membership;
pub mod postgres;
pub mod provisioning;
pub mod session_policy;

/// Internal carrier identifier; no public transport format or generation algorithm implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CarrierId(NonZeroU128);

impl CarrierId {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }

    pub fn value(self) -> u128 {
        self.0.get()
    }
}

/// Internal facility identifier, distinct from a carrier identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FacilityId(NonZeroU128);

impl FacilityId {
    pub fn new(value: u128) -> Option<Self> {
        NonZeroU128::new(value).map(Self)
    }

    pub fn value(self) -> u128 {
        self.0.get()
    }
}

/// A facility's association is immutable through this model API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facility {
    id: FacilityId,
    carrier: Option<CarrierId>,
}

impl Facility {
    pub fn id(&self) -> FacilityId {
        self.id
    }

    pub fn carrier(&self) -> Option<CarrierId> {
        self.carrier
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    DuplicateCarrier,
    DuplicateFacility,
    UnknownCarrier,
}

/// In-memory domain model, not a production repository or a permission decision.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrganizationModel {
    carriers: BTreeSet<CarrierId>,
    facilities: BTreeMap<FacilityId, Facility>,
}

impl OrganizationModel {
    /// Register an internal carrier reference without granting any access rights.
    ///
    /// A facility identifier cannot be used as a carrier identifier:
    /// ```compile_fail
    /// use kitaconnect_organization_config::{FacilityId, OrganizationModel};
    /// let mut model = OrganizationModel::default();
    /// model.register_carrier(FacilityId::new(1).unwrap());
    /// ```
    pub fn register_carrier(&mut self, id: CarrierId) -> Result<(), ModelError> {
        if !self.carriers.insert(id) {
            return Err(ModelError::DuplicateCarrier);
        }
        Ok(())
    }

    /// Caller must enforce current authorization before using this internal operation.
    pub fn register_facility(
        &mut self,
        id: FacilityId,
        carrier: Option<CarrierId>,
    ) -> Result<(), ModelError> {
        if self.facilities.contains_key(&id) {
            return Err(ModelError::DuplicateFacility);
        }
        if let Some(carrier) = carrier
            && !self.carriers.contains(&carrier)
        {
            return Err(ModelError::UnknownCarrier);
        }
        self.facilities.insert(id, Facility { id, carrier });
        Ok(())
    }

    pub fn contains_carrier(&self, id: CarrierId) -> bool {
        self.carriers.contains(&id)
    }

    /// Internal lookup; existence and association do not authorize a user to read.
    pub fn facility(&self, id: FacilityId) -> Option<&Facility> {
        self.facilities.get(&id)
    }
}
