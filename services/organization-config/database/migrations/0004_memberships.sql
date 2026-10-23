-- Internal read foundation only. Positive writes are isolated administrator fixtures.
SET LOCAL ROLE kc_org_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184004,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM organization_config.schema_versions) IS DISTINCT FROM ARRAY[1,2,3] THEN
        RAISE EXCEPTION 'memberships require migration versions 1 to 3';
    END IF;
END $$;
CREATE TABLE organization_config.memberships (
    identity_ref organization_config.internal_id NOT NULL,
    facility_id organization_config.internal_id NOT NULL REFERENCES organization_config.facilities(id),
    version numeric NOT NULL CHECK (version=trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    active boolean NOT NULL DEFAULT false,
    PRIMARY KEY (identity_ref,facility_id)
);
CREATE FUNCTION organization_config.guard_membership() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF TG_OP='INSERT' THEN
        IF NEW.version<>1 THEN RAISE EXCEPTION 'invalid membership initial version' USING ERRCODE='23514'; END IF;
    ELSIF (NEW.identity_ref,NEW.facility_id) IS DISTINCT FROM (OLD.identity_ref,OLD.facility_id)
        OR NEW.version<>OLD.version+1 THEN
        RAISE EXCEPTION 'invalid membership transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION organization_config.guard_membership() FROM PUBLIC;
CREATE TRIGGER membership_guard BEFORE INSERT OR UPDATE ON organization_config.memberships
FOR EACH ROW EXECUTE FUNCTION organization_config.guard_membership();
ALTER TABLE organization_config.memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_config.memberships FORCE ROW LEVEL SECURITY;
REVOKE ALL ON organization_config.memberships FROM PUBLIC;
GRANT SELECT ON organization_config.memberships TO kc_org_runtime;
-- No cross-service FK, runtime writes or permissive policy.
INSERT INTO organization_config.schema_versions VALUES(4);
