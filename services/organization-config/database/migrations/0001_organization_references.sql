-- Apply with psql --single-transaction --set=ON_ERROR_STOP=1.
-- No application-start migration and no implicit adoption of existing objects.
SET LOCAL ROLE kc_org_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184004, 1);

ALTER DEFAULT PRIVILEGES IN SCHEMA organization_config REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
ALTER DEFAULT PRIVILEGES IN SCHEMA organization_config REVOKE USAGE ON TYPES FROM PUBLIC;

CREATE DOMAIN organization_config.internal_id AS numeric
    CHECK (VALUE > 0
       AND VALUE <= 340282366920938463463374607431768211455
       AND VALUE = pg_catalog.trunc(VALUE));
REVOKE ALL ON DOMAIN organization_config.internal_id FROM PUBLIC;
GRANT USAGE ON DOMAIN organization_config.internal_id TO kc_org_runtime;

CREATE TABLE organization_config.carriers (
    id organization_config.internal_id PRIMARY KEY
);
CREATE TABLE organization_config.facilities (
    id organization_config.internal_id PRIMARY KEY,
    carrier_id organization_config.internal_id
        REFERENCES organization_config.carriers (id) ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE INDEX facilities_carrier_id ON organization_config.facilities (carrier_id);

ALTER TABLE organization_config.carriers ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_config.carriers FORCE ROW LEVEL SECURITY;
ALTER TABLE organization_config.facilities ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_config.facilities FORCE ROW LEVEL SECURITY;
REVOKE ALL ON organization_config.carriers, organization_config.facilities FROM PUBLIC;
GRANT USAGE ON SCHEMA organization_config TO kc_org_runtime;
GRANT SELECT, INSERT ON organization_config.carriers, organization_config.facilities TO kc_org_runtime;
-- Intentionally no policies: fail closed until the current identity/policy boundary exists.

CREATE TABLE organization_config.schema_versions (
    version integer PRIMARY KEY CHECK (version > 0)
);
REVOKE ALL ON organization_config.schema_versions FROM PUBLIC;
INSERT INTO organization_config.schema_versions (version) VALUES (1);
