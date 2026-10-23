-- Apply after 0001 with psql --single-transaction --set=ON_ERROR_STOP=1.
SET LOCAL ROLE kc_org_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184004, 1);
DO $$
BEGIN
    IF (SELECT pg_catalog.array_agg(version ORDER BY version) FROM organization_config.schema_versions) IS DISTINCT FROM ARRAY[1] THEN
        RAISE EXCEPTION 'facility schema registry requires migration version 1';
    END IF;
END $$;

CREATE TABLE organization_config.facility_schemas (
    facility_id organization_config.internal_id PRIMARY KEY
        REFERENCES organization_config.facilities (id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    schema_name text COLLATE "C" NOT NULL UNIQUE
        CHECK (schema_name ~ '^facility_[0-9a-f]{32}$')
);
ALTER TABLE organization_config.facility_schemas ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_config.facility_schemas FORCE ROW LEVEL SECURITY;
REVOKE ALL ON organization_config.facility_schemas FROM PUBLIC;
GRANT USAGE ON SCHEMA organization_config TO kc_facility_provisioner;
GRANT USAGE ON DOMAIN organization_config.internal_id TO kc_facility_provisioner;
GRANT SELECT (id) ON organization_config.facilities TO kc_facility_provisioner;
CREATE POLICY facility_maintenance_identifiers ON organization_config.facilities
    FOR SELECT TO kc_facility_provisioner USING (true);
GRANT SELECT, INSERT ON organization_config.facility_schemas TO kc_facility_provisioner;
CREATE POLICY facility_registry_maintenance_read ON organization_config.facility_schemas
    FOR SELECT TO kc_facility_provisioner USING (true);
CREATE POLICY facility_registry_maintenance_insert ON organization_config.facility_schemas
    FOR INSERT TO kc_facility_provisioner WITH CHECK (true);
INSERT INTO organization_config.schema_versions (version) VALUES (2);
