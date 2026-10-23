-- Apply as migration role after 0001 and 0002 in a single transaction.
SET LOCAL ROLE kc_org_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184004, 1);
DO $$ BEGIN
    IF (SELECT pg_catalog.array_agg(version ORDER BY version) FROM organization_config.schema_versions) IS DISTINCT FROM ARRAY[1,2] THEN
        RAISE EXCEPTION 'session policies require migration versions 1 and 2';
    END IF;
END $$;

CREATE TABLE organization_config.facility_session_policies (
    facility_id organization_config.internal_id PRIMARY KEY REFERENCES organization_config.facilities(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    version numeric NOT NULL CHECK (version = trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    idle_seconds numeric NOT NULL CHECK (idle_seconds = trunc(idle_seconds) AND idle_seconds BETWEEN 1 AND 900),
    absolute_seconds numeric NOT NULL CHECK (absolute_seconds = trunc(absolute_seconds) AND absolute_seconds BETWEEN 1 AND 28800),
    step_up_seconds numeric NOT NULL CHECK (step_up_seconds = trunc(step_up_seconds) AND step_up_seconds BETWEEN 1 AND 300),
    CHECK (idle_seconds <= absolute_seconds AND step_up_seconds <= absolute_seconds)
);
CREATE FUNCTION organization_config.guard_session_policy() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path = pg_catalog AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.version <> 1 THEN
            RAISE EXCEPTION 'invalid session policy initial version' USING ERRCODE = '23514';
        END IF;
    ELSIF NEW.facility_id IS DISTINCT FROM OLD.facility_id OR NEW.version <> OLD.version + 1 THEN
        RAISE EXCEPTION 'invalid session policy transition' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION organization_config.guard_session_policy() FROM PUBLIC;
CREATE TRIGGER session_policy_guard BEFORE INSERT OR UPDATE ON organization_config.facility_session_policies
    FOR EACH ROW EXECUTE FUNCTION organization_config.guard_session_policy();
ALTER TABLE organization_config.facility_session_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_config.facility_session_policies FORCE ROW LEVEL SECURITY;
REVOKE ALL ON organization_config.facility_session_policies FROM PUBLIC;
GRANT SELECT, INSERT ON organization_config.facility_session_policies TO kc_org_runtime;
GRANT UPDATE (version, idle_seconds, absolute_seconds, step_up_seconds) ON organization_config.facility_session_policies TO kc_org_runtime;
-- No permissive runtime policies, implicit defaults or session creation.
INSERT INTO organization_config.schema_versions VALUES (3);
