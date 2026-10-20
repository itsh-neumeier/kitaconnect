-- Test-only artifact. Run in one transaction in the isolated PostgreSQL fixture.
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184005, 2);

DO $$
DECLARE
    service text;
    facility integer;
    fixture_schema text;
    login_name text;
BEGIN
    ASSERT current_database() = 'kc_facility_schema_test';
    FOR facility IN 1..2 LOOP
        fixture_schema := 'facility_' || lpad(to_hex(facility), 32, '0');
        ASSERT EXISTS (
            SELECT FROM organization_config.facility_schemas r
            JOIN pg_namespace n ON n.nspname = r.schema_name
            WHERE r.facility_id = facility AND r.schema_name = fixture_schema
                AND pg_get_userbyid(n.nspowner) = 'kc_facility_schema_owner'
        ), 'registered schema required';
    END LOOP;
    EXECUTE format('REVOKE TEMPORARY, CREATE ON DATABASE %I FROM PUBLIC', current_database());
    CREATE ROLE kc_test_unmapped LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
    GRANT USAGE ON SCHEMA facility_00000000000000000000000000000001, facility_00000000000000000000000000000002 TO kc_test_unmapped;
    FOREACH service IN ARRAY ARRAY['care', 'meals'] LOOP
        EXECUTE format('CREATE ROLE %I NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS', 'kc_test_' || service || '_owner');
        EXECUTE format('CREATE ROLE %I LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS', 'kc_test_' || service || '_migrator');
        EXECUTE format('GRANT %I TO %I WITH ADMIN FALSE, INHERIT FALSE, SET TRUE', 'kc_test_' || service || '_owner', 'kc_test_' || service || '_migrator');
        FOR facility IN 1..2 LOOP
            fixture_schema := 'facility_' || lpad(to_hex(facility), 32, '0');
            login_name := 'kc_test_' || service || '_' || facility;
            EXECUTE format('CREATE ROLE %I LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS', login_name);
            EXECUTE format('GRANT USAGE ON SCHEMA %I TO %I', fixture_schema, login_name);
            EXECUTE format('GRANT USAGE, CREATE ON SCHEMA %I TO %I', fixture_schema, 'kc_test_' || service || '_owner');
        END LOOP;
    END LOOP;
END $$;

-- Each trusted migration runs as its own owner, never as an application login.
SET LOCAL ROLE kc_test_care_owner;
ALTER DEFAULT PRIVILEGES REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
DO $$
DECLARE
    facility integer;
    fixture_schema text;
    login_name text;
BEGIN
    FOR facility IN 1..2 LOOP
        fixture_schema := 'facility_' || lpad(to_hex(facility), 32, '0');
        login_name := 'kc_test_care_' || facility;
        EXECUTE format('CREATE TABLE %I.care_probe (id integer PRIMARY KEY, facility_id numeric NOT NULL, payload integer NOT NULL)', fixture_schema);
        EXECUTE format('ALTER TABLE %I.care_probe ENABLE ROW LEVEL SECURITY', fixture_schema);
        EXECUTE format('ALTER TABLE %I.care_probe FORCE ROW LEVEL SECURITY', fixture_schema);
        EXECUTE format('CREATE POLICY fixture_identity ON %I.care_probe TO %I USING (facility_id = %L::numeric AND CURRENT_USER = %L) WITH CHECK (facility_id = %L::numeric AND CURRENT_USER = %L)', fixture_schema, login_name, facility, login_name, facility, login_name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I.care_probe TO %I', fixture_schema, login_name);
        EXECUTE format('CREATE FUNCTION %I.care_probe_count() RETURNS bigint LANGUAGE sql STABLE SECURITY INVOKER SET search_path = pg_catalog, pg_temp AS %L', fixture_schema, format('SELECT pg_catalog.count(*) FROM %I.care_probe', fixture_schema));
        EXECUTE format('REVOKE ALL ON FUNCTION %I.care_probe_count() FROM PUBLIC', fixture_schema);
        EXECUTE format('GRANT EXECUTE ON FUNCTION %I.care_probe_count() TO %I', fixture_schema, login_name);
    END LOOP;
END $$;

SET LOCAL ROLE kc_test_meals_owner;
ALTER DEFAULT PRIVILEGES REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
DO $$
DECLARE
    facility integer;
    fixture_schema text;
    login_name text;
BEGIN
    FOR facility IN 1..2 LOOP
        fixture_schema := 'facility_' || lpad(to_hex(facility), 32, '0');
        login_name := 'kc_test_meals_' || facility;
        EXECUTE format('CREATE TABLE %I.meals_probe (id integer PRIMARY KEY, facility_id numeric NOT NULL, payload integer NOT NULL)', fixture_schema);
        EXECUTE format('ALTER TABLE %I.meals_probe ENABLE ROW LEVEL SECURITY', fixture_schema);
        EXECUTE format('ALTER TABLE %I.meals_probe FORCE ROW LEVEL SECURITY', fixture_schema);
        EXECUTE format('CREATE POLICY fixture_identity ON %I.meals_probe TO %I USING (facility_id = %L::numeric AND CURRENT_USER = %L) WITH CHECK (facility_id = %L::numeric AND CURRENT_USER = %L)', fixture_schema, login_name, facility, login_name, facility, login_name);
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %I.meals_probe TO %I', fixture_schema, login_name);
        EXECUTE format('CREATE FUNCTION %I.meals_probe_count() RETURNS bigint LANGUAGE sql STABLE SECURITY INVOKER SET search_path = pg_catalog, pg_temp AS %L', fixture_schema, format('SELECT pg_catalog.count(*) FROM %I.meals_probe', fixture_schema));
        EXECUTE format('REVOKE ALL ON FUNCTION %I.meals_probe_count() FROM PUBLIC', fixture_schema);
        EXECUTE format('GRANT EXECUTE ON FUNCTION %I.meals_probe_count() TO %I', fixture_schema, login_name);
    END LOOP;
END $$;

SET LOCAL ROLE NONE;
DO $$
DECLARE
    facility integer;
    service text;
    fixture_schema text;
BEGIN
    FOR facility IN 1..2 LOOP
        fixture_schema := 'facility_' || lpad(to_hex(facility), 32, '0');
        FOREACH service IN ARRAY ARRAY['care', 'meals'] LOOP
            EXECUTE format('REVOKE CREATE ON SCHEMA %I FROM %I', fixture_schema, 'kc_test_' || service || '_owner');
            -- Only the fixture administrator inserts the deliberately foreign row.
            EXECUTE format('INSERT INTO %I.%I VALUES (1, %L, 100), (2, %L, 900)', fixture_schema, service || '_probe', facility, 3 - facility);
        END LOOP;
    END LOOP;
END $$;

-- A hostile writable namespace is modelled by the administrator, not granted to runtime.
CREATE SCHEMA kc_test_shadow;
CREATE TABLE kc_test_shadow.care_probe (payload integer);
INSERT INTO kc_test_shadow.care_probe VALUES (999);
CREATE FUNCTION kc_test_shadow.care_probe_count() RETURNS bigint LANGUAGE sql AS 'SELECT 999::bigint';
REVOKE ALL ON FUNCTION kc_test_shadow.care_probe_count() FROM PUBLIC;
GRANT USAGE ON SCHEMA kc_test_shadow TO kc_test_care_1;
GRANT SELECT ON kc_test_shadow.care_probe TO kc_test_care_1;
GRANT EXECUTE ON FUNCTION kc_test_shadow.care_probe_count() TO kc_test_care_1;
