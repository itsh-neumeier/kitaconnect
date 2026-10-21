use kitaconnect_organization_config::{CarrierId, FacilityId, ModelError, postgres};
use postgres::StorageError;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{Connection, PgConnection};

fn carrier(value: u128) -> CarrierId {
    CarrierId::new(value).unwrap()
}

fn facility(value: u128) -> FacilityId {
    FacilityId::new(value).unwrap()
}

async fn connect(user: &str) -> PgConnection {
    let socket = std::env::var("KC_ORG_TEST_SOCKET").expect("isolated test socket is required");
    PgConnection::connect_with(
        &PgConnectOptions::new()
            .socket(socket)
            .username(user)
            .database("kc_org_adapter_test")
            .ssl_mode(PgSslMode::Disable), // Unix socket in the ephemeral, network-disabled test container only.
    )
    .await
    .expect("connect to the isolated PostgreSQL fixture")
}

fn assert_permission_denied(error: StorageError) {
    assert_eq!(error.to_string(), "organization storage operation failed");
    match error {
        StorageError::Database(error) => {
            assert_eq!(
                error.as_database_error().unwrap().code().as_deref(),
                Some("42501")
            );
        }
        other => panic!("expected database permission error, received {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "run explicitly through scripts/test-organization-db.mjs with the isolated PostgreSQL fixture"]
async fn storage_adapter_controls_references() {
    let phase = std::env::var("KC_ORG_TEST_PHASE").expect("test phase is required");
    assert!(matches!(phase.as_str(), "before" | "after"));
    // The fixture administrator is used only to exercise storage behind fail-closed RLS.
    // It is not an application identity or a proposed production permission.
    let mut admin = connect("postgres").await;
    // A successful SQL command is not evidence that a BEFORE trigger inserted a row.
    // Run in both phases; all fixture DDL is removed again before the normal cases.
    sqlx::raw_sql("CREATE FUNCTION organization_config.suppress_registration() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$; CREATE TRIGGER suppress_registration BEFORE INSERT ON organization_config.carriers FOR EACH ROW EXECUTE FUNCTION organization_config.suppress_registration(); CREATE TRIGGER suppress_registration BEFORE INSERT ON organization_config.facilities FOR EACH ROW EXECUTE FUNCTION organization_config.suppress_registration();")
        .execute(&mut admin).await.unwrap();
    assert!(matches!(
        postgres::register_carrier(&mut admin, carrier(88)).await,
        Err(StorageError::WriteNotConfirmed)
    ));
    assert!(matches!(
        postgres::register_facility(&mut admin, facility(88), None).await,
        Err(StorageError::WriteNotConfirmed)
    ));
    assert!(
        !postgres::contains_carrier(&mut admin, carrier(88))
            .await
            .unwrap()
    );
    assert_eq!(
        postgres::facility(&mut admin, facility(88)).await.unwrap(),
        None
    );
    sqlx::raw_sql("DROP TRIGGER suppress_registration ON organization_config.carriers; DROP TRIGGER suppress_registration ON organization_config.facilities; DROP FUNCTION organization_config.suppress_registration();")
        .execute(&mut admin).await.unwrap();
    if phase == "before" {
        postgres::register_carrier(&mut admin, carrier(1))
            .await
            .unwrap();
        postgres::register_carrier(&mut admin, carrier(2))
            .await
            .unwrap();
        postgres::register_carrier(&mut admin, carrier(u128::MAX))
            .await
            .unwrap();
        for (id, carrier_id) in [
            (1, Some(carrier(1))),
            (2, Some(carrier(2))),
            (3, None),
            (u128::MAX, Some(carrier(u128::MAX))),
        ] {
            postgres::register_facility(&mut admin, facility(id), carrier_id)
                .await
                .unwrap();
        }
        assert!(matches!(
            postgres::register_carrier(&mut admin, carrier(1)).await,
            Err(StorageError::Model(ModelError::DuplicateCarrier))
        ));
        assert!(matches!(
            postgres::register_facility(&mut admin, facility(1), Some(carrier(2))).await,
            Err(StorageError::Model(ModelError::DuplicateFacility))
        ));
        assert!(matches!(
            postgres::register_facility(&mut admin, facility(9), Some(carrier(9))).await,
            Err(StorageError::Model(ModelError::UnknownCarrier))
        ));
        assert_eq!(
            postgres::facility(&mut admin, facility(9)).await.unwrap(),
            None
        );

        let mut transaction = admin.begin().await.unwrap();
        postgres::register_carrier(&mut transaction, carrier(90))
            .await
            .unwrap();
        assert!(matches!(
            postgres::register_facility(&mut transaction, facility(90), Some(carrier(99))).await,
            Err(StorageError::Model(ModelError::UnknownCarrier))
        ));
        transaction.rollback().await.unwrap();
        assert!(
            !postgres::contains_carrier(&mut admin, carrier(90))
                .await
                .unwrap()
        );
        assert_eq!(
            postgres::facility(&mut admin, facility(90)).await.unwrap(),
            None
        );

        // SQL permits a mathematically integral value with a scale; decoding must remain lossless.
        sqlx::query("INSERT INTO organization_config.carriers (id) VALUES (10.000)")
            .execute(&mut admin)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO organization_config.facilities (id, carrier_id) VALUES (10.000, 10.000)",
        )
        .execute(&mut admin)
        .await
        .unwrap();
    }

    for (id, carrier_id) in [
        (1, Some(carrier(1))),
        (2, Some(carrier(2))),
        (3, None),
        (10, Some(carrier(10))),
        (u128::MAX, Some(carrier(u128::MAX))),
    ] {
        let record = postgres::facility(&mut admin, facility(id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.id(), facility(id));
        assert_eq!(record.carrier(), carrier_id);
    }
    assert!(
        postgres::contains_carrier(&mut admin, carrier(1))
            .await
            .unwrap()
    );
    assert!(
        postgres::contains_carrier(&mut admin, carrier(u128::MAX))
            .await
            .unwrap()
    );
    assert!(
        !postgres::contains_carrier(&mut admin, carrier(42))
            .await
            .unwrap()
    );
    assert_eq!(
        postgres::facility(&mut admin, facility(42)).await.unwrap(),
        None
    );

    let mut runtime = connect("kc_org_adapter_runtime").await;
    assert!(
        !postgres::contains_carrier(&mut runtime, carrier(1))
            .await
            .unwrap()
    );
    assert_eq!(
        postgres::facility(&mut runtime, facility(1)).await.unwrap(),
        None
    );
    assert_permission_denied(
        postgres::register_carrier(&mut runtime, carrier(99))
            .await
            .unwrap_err(),
    );
    assert_permission_denied(
        postgres::register_facility(&mut runtime, facility(99), None)
            .await
            .unwrap_err(),
    );
    let user: String = sqlx::query_scalar("SELECT current_user::text")
        .fetch_one(&mut runtime)
        .await
        .unwrap();
    assert_eq!(user, "kc_org_adapter_runtime");
    assert!(
        !postgres::contains_carrier(&mut admin, carrier(99))
            .await
            .unwrap()
    );
    assert_eq!(
        postgres::facility(&mut admin, facility(99)).await.unwrap(),
        None
    );
}
