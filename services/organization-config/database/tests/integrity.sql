-- Only the ephemeral test administrator seeds data, never the runtime identity.
INSERT INTO organization_config.carriers (id) VALUES (1), (2);
INSERT INTO organization_config.facilities (id, carrier_id) VALUES (1, 1), (2, 2), (3, NULL);

DO $$
BEGIN
    BEGIN
        INSERT INTO organization_config.carriers VALUES (1);
        RAISE EXCEPTION 'duplicate carrier accepted';
    EXCEPTION WHEN unique_violation THEN NULL;
    END;
    BEGIN
        INSERT INTO organization_config.facilities VALUES (1, 2);
        RAISE EXCEPTION 'duplicate facility replaced association';
    EXCEPTION WHEN unique_violation THEN NULL;
    END;
    BEGIN
        INSERT INTO organization_config.facilities VALUES (4, 999);
        RAISE EXCEPTION 'unknown carrier accepted';
    EXCEPTION WHEN foreign_key_violation THEN NULL;
    END;
    ASSERT (SELECT carrier_id = 1 FROM organization_config.facilities WHERE id = 1);
    ASSERT (SELECT count(*) = 3 FROM organization_config.facilities);
END $$;

DO $$
DECLARE invalid numeric;
BEGIN
    FOREACH invalid IN ARRAY ARRAY[0, -1, 1.5, 340282366920938463463374607431768211456, 'NaN'::numeric, 'Infinity'::numeric] LOOP
        BEGIN
            INSERT INTO organization_config.carriers VALUES (invalid);
            RAISE EXCEPTION 'invalid identifier accepted: %', invalid;
        EXCEPTION WHEN check_violation THEN NULL;
        END;
    END LOOP;
    -- Boundary is lossless, then remove the extra test reference as test administrator.
    INSERT INTO organization_config.carriers VALUES (340282366920938463463374607431768211455);
    ASSERT (SELECT id::text = '340282366920938463463374607431768211455'
            FROM organization_config.carriers WHERE id = 340282366920938463463374607431768211455);
    DELETE FROM organization_config.carriers WHERE id = 340282366920938463463374607431768211455;
END $$;
