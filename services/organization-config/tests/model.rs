use kitaconnect_organization_config::{CarrierId, FacilityId, ModelError, OrganizationModel};

fn carrier(value: u128) -> CarrierId {
    CarrierId::new(value).unwrap()
}

fn facility(value: u128) -> FacilityId {
    FacilityId::new(value).unwrap()
}

fn model_with_two_carriers() -> OrganizationModel {
    let mut model = OrganizationModel::default();
    model.register_carrier(carrier(1)).unwrap();
    model.register_carrier(carrier(2)).unwrap();
    model
        .register_facility(facility(1), Some(carrier(1)))
        .unwrap();
    model
        .register_facility(facility(2), Some(carrier(2)))
        .unwrap();
    model.register_facility(facility(3), None).unwrap();
    model
}

#[test]
fn separates_two_carriers_and_an_independent_facility() {
    let model = model_with_two_carriers();
    assert_eq!(
        model.facility(facility(1)).unwrap().carrier(),
        Some(carrier(1))
    );
    assert_eq!(
        model.facility(facility(2)).unwrap().carrier(),
        Some(carrier(2))
    );
    assert_eq!(model.facility(facility(3)).unwrap().carrier(), None);
    assert_eq!(model.facility(facility(3)).unwrap().id(), facility(3));
    assert!(model.contains_carrier(carrier(1)));
    assert!(model.contains_carrier(carrier(2)));
}

#[test]
fn rejects_zero_and_preserves_full_identifier_range() {
    assert_eq!(CarrierId::new(0), None);
    assert_eq!(FacilityId::new(0), None);
    for value in [1, u128::MAX] {
        assert_eq!(carrier(value).value(), value);
        assert_eq!(facility(value).value(), value);
    }
}

#[test]
fn rejects_duplicate_carrier_without_changing_state() {
    let mut model = model_with_two_carriers();
    let before = model.clone();
    assert_eq!(
        model.register_carrier(carrier(1)),
        Err(ModelError::DuplicateCarrier)
    );
    assert_eq!(model, before);
}

#[test]
fn rejects_unknown_carrier_without_changing_state() {
    let mut model = model_with_two_carriers();
    let before = model.clone();
    assert_eq!(
        model.register_facility(facility(4), Some(carrier(9))),
        Err(ModelError::UnknownCarrier)
    );
    assert_eq!(model, before);
    assert_eq!(model.facility(facility(4)), None);
}

#[test]
fn rejects_duplicate_facility_and_all_reassignment_attempts() {
    let mut model = model_with_two_carriers();
    let before = model.clone();
    for target in [Some(carrier(1)), Some(carrier(2)), Some(carrier(9)), None] {
        assert_eq!(
            model.register_facility(facility(1), target),
            Err(ModelError::DuplicateFacility)
        );
        assert_eq!(model, before);
    }
}

#[test]
fn cannot_attach_an_existing_independent_facility_by_registering_again() {
    let mut model = model_with_two_carriers();
    let before = model.clone();
    assert_eq!(
        model.register_facility(facility(3), Some(carrier(1))),
        Err(ModelError::DuplicateFacility)
    );
    assert_eq!(model, before);
}

#[test]
fn missing_references_do_not_resolve_to_another_organization() {
    let model = model_with_two_carriers();
    assert!(!model.contains_carrier(carrier(99)));
    assert_eq!(model.facility(facility(99)), None);
}

#[test]
fn accepts_independent_facility_without_any_carriers() {
    let mut model = OrganizationModel::default();
    model.register_facility(facility(u128::MAX), None).unwrap();
    assert_eq!(model.facility(facility(u128::MAX)).unwrap().carrier(), None);
}
