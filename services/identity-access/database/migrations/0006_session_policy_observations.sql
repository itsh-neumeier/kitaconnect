-- Apply after 0001..0005 as the migration role in one transaction.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5] THEN
        RAISE EXCEPTION 'session observations require migration versions 1 to 5';
    END IF;
END $$;
CREATE TABLE identity_access.session_policy_observations (
    session_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.session_login_bindings(session_id),
    version numeric NOT NULL CHECK (version=trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    raw_idle integer NOT NULL CHECK (raw_idle BETWEEN 1 AND 900),
    raw_absolute integer NOT NULL CHECK (raw_absolute BETWEEN 1 AND 28800),
    raw_step_up integer NOT NULL CHECK (raw_step_up BETWEEN 1 AND 300),
    idle_cap integer NOT NULL CHECK (idle_cap BETWEEN 1 AND raw_idle),
    absolute_cap integer NOT NULL CHECK (absolute_cap BETWEEN 1 AND raw_absolute),
    step_up_cap integer NOT NULL CHECK (step_up_cap BETWEEN 1 AND raw_step_up),
    mfa_required boolean NOT NULL,
    CHECK (raw_idle<=raw_absolute AND raw_step_up<=raw_absolute),
    CHECK (idle_cap<=absolute_cap AND step_up_cap<=absolute_cap)
);
CREATE FUNCTION identity_access.guard_session_policy_observation() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF NEW.session_id IS DISTINCT FROM OLD.session_id OR NEW.version<OLD.version
        OR (NEW.version=OLD.version AND (NEW.raw_idle,NEW.raw_absolute,NEW.raw_step_up) IS DISTINCT FROM (OLD.raw_idle,OLD.raw_absolute,OLD.raw_step_up))
        OR NEW.idle_cap>OLD.idle_cap OR NEW.absolute_cap>OLD.absolute_cap OR NEW.step_up_cap>OLD.step_up_cap
        OR (OLD.mfa_required AND NOT NEW.mfa_required) THEN
        RAISE EXCEPTION 'invalid session policy observation transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_session_policy_observation() FROM PUBLIC;
CREATE TRIGGER session_policy_observation_guard BEFORE UPDATE ON identity_access.session_policy_observations
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_session_policy_observation();
ALTER TABLE identity_access.session_policy_observations ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_policy_observations FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.session_policy_observations FROM PUBLIC;
GRANT SELECT,INSERT ON identity_access.session_policy_observations TO kc_identity_runtime;
GRANT UPDATE (version,raw_idle,raw_absolute,raw_step_up,idle_cap,absolute_cap,step_up_cap,mfa_required)
ON identity_access.session_policy_observations TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES (6);
