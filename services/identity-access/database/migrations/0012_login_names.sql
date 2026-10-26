-- Own local login name only. No registration, membership or authentication grant.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6,7,8,9,10,11] THEN
        RAISE EXCEPTION 'login names require migration versions 1 to 11';
    END IF;
END $$;
CREATE TABLE identity_access.login_names (
    name bytea PRIMARY KEY CHECK(octet_length(name) BETWEEN 1 AND 256),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.identities(id),
    state text COLLATE "C" NOT NULL CHECK(state IN ('active','revoked'))
);
CREATE FUNCTION identity_access.guard_login_name() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF TG_OP='DELETE' THEN
        RAISE EXCEPTION 'login name deletion is not revocation' USING ERRCODE='23514';
    ELSIF (NEW.name,NEW.identity_id) IS DISTINCT FROM (OLD.name,OLD.identity_id)
        OR (OLD.state='revoked' AND NEW.state<>'revoked') THEN
        RAISE EXCEPTION 'invalid login name transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_login_name() FROM PUBLIC;
CREATE TRIGGER login_name_guard BEFORE UPDATE OR DELETE ON identity_access.login_names
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_login_name();
ALTER TABLE identity_access.login_names ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.login_names FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.login_names FROM PUBLIC;
GRANT SELECT ON identity_access.login_names TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES (12);
