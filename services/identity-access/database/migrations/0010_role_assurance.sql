-- Complete scoped role classifications only; no business grants or default roles.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6,7,8,9] THEN
        RAISE EXCEPTION 'role assurance requires migration versions 1 to 9';
    END IF;
END $$;
CREATE TABLE identity_access.role_assurance_scopes (
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.identities(id),
    facility_id identity_access.internal_id NOT NULL,
    version numeric NOT NULL CHECK(version=trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    state text COLLATE "C" NOT NULL DEFAULT 'revoked' CHECK(state IN ('active','revoked')),
    document text NOT NULL CHECK(octet_length(document) BETWEEN 1 AND 65536),
    PRIMARY KEY(identity_id,facility_id)
);
CREATE FUNCTION identity_access.guard_role_assurance() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF TG_OP='DELETE' THEN
        RAISE EXCEPTION 'role assurance deletion is not revocation' USING ERRCODE='23514';
    ELSIF TG_OP='INSERT' THEN
        IF NEW.version<>1 THEN
            RAISE EXCEPTION 'invalid initial role assurance version' USING ERRCODE='23514';
        END IF;
    ELSIF (NEW.identity_id,NEW.facility_id) IS DISTINCT FROM (OLD.identity_id,OLD.facility_id)
        OR NEW.version<>OLD.version+1 OR OLD.state='revoked' THEN
        RAISE EXCEPTION 'invalid role assurance transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_role_assurance() FROM PUBLIC;
CREATE TRIGGER role_assurance_guard BEFORE INSERT OR UPDATE OR DELETE ON identity_access.role_assurance_scopes
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_role_assurance();
ALTER TABLE identity_access.role_assurance_scopes ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.role_assurance_scopes FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.role_assurance_scopes FROM PUBLIC;
GRANT SELECT ON identity_access.role_assurance_scopes TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES (10);
