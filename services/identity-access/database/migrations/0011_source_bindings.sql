-- Complete local source receipts; no user/domain grants or remote foreign keys.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6,7,8,9,10] THEN
        RAISE EXCEPTION 'source bindings require migration versions 1 to 10';
    END IF;
END $$;
CREATE TABLE identity_access.flow_source_bindings (
    flow_fingerprint bytea PRIMARY KEY REFERENCES identity_access.login_flows(fingerprint) ON DELETE CASCADE,
    membership_version numeric NOT NULL CHECK(membership_version=trunc(membership_version) AND membership_version BETWEEN 1 AND 18446744073709551615),
    policy_version numeric NOT NULL CHECK(policy_version=trunc(policy_version) AND policy_version BETWEEN 1 AND 18446744073709551615),
    rights_version numeric NOT NULL CHECK(rights_version=trunc(rights_version) AND rights_version BETWEEN 1 AND 18446744073709551615),
    raw_idle numeric NOT NULL CHECK(raw_idle=trunc(raw_idle) AND raw_idle BETWEEN 1 AND 900),
    raw_absolute numeric NOT NULL CHECK(raw_absolute=trunc(raw_absolute) AND raw_absolute BETWEEN 1 AND 28800),
    raw_step_up numeric NOT NULL CHECK(raw_step_up=trunc(raw_step_up) AND raw_step_up BETWEEN 1 AND 300),
    raw_mfa_required boolean NOT NULL,
    CHECK(raw_idle<=raw_absolute AND raw_step_up<=raw_absolute)
);
CREATE TABLE identity_access.session_source_observations (
    session_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.session_login_bindings(session_id),
    membership_version numeric NOT NULL CHECK(membership_version=trunc(membership_version) AND membership_version BETWEEN 1 AND 18446744073709551615),
    policy_version numeric NOT NULL CHECK(policy_version=trunc(policy_version) AND policy_version BETWEEN 1 AND 18446744073709551615),
    rights_version numeric NOT NULL CHECK(rights_version=trunc(rights_version) AND rights_version BETWEEN 1 AND 18446744073709551615),
    raw_idle numeric NOT NULL CHECK(raw_idle=trunc(raw_idle) AND raw_idle BETWEEN 1 AND 900),
    raw_absolute numeric NOT NULL CHECK(raw_absolute=trunc(raw_absolute) AND raw_absolute BETWEEN 1 AND 28800),
    raw_step_up numeric NOT NULL CHECK(raw_step_up=trunc(raw_step_up) AND raw_step_up BETWEEN 1 AND 300),
    raw_mfa_required boolean NOT NULL,
    CHECK(raw_idle<=raw_absolute AND raw_step_up<=raw_absolute)
);
CREATE FUNCTION identity_access.guard_flow_sources() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$ BEGIN
    IF TG_OP='UPDATE' OR NOT EXISTS(SELECT FROM identity_access.login_flows f WHERE f.fingerprint=NEW.flow_fingerprint
        AND (f.policy_version,f.idle_seconds,f.absolute_seconds,f.step_up_seconds)=(NEW.policy_version,NEW.raw_idle,NEW.raw_absolute,NEW.raw_step_up)
        AND (NOT NEW.raw_mfa_required OR f.mfa_required)) THEN
        RAISE EXCEPTION 'invalid flow source binding' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION identity_access.guard_session_sources() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$ BEGIN
    IF TG_OP='UPDATE' THEN
        IF NEW.session_id<>OLD.session_id OR NEW.membership_version<>OLD.membership_version
            OR NEW.policy_version<OLD.policy_version OR NEW.rights_version<OLD.rights_version
            OR (NEW.policy_version=OLD.policy_version AND (NEW.raw_idle,NEW.raw_absolute,NEW.raw_step_up) IS DISTINCT FROM (OLD.raw_idle,OLD.raw_absolute,OLD.raw_step_up))
            OR (NEW.rights_version=OLD.rights_version AND NEW.raw_mfa_required<>OLD.raw_mfa_required) THEN
            RAISE EXCEPTION 'invalid source observation transition' USING ERRCODE='23514';
        END IF;
    END IF;
    IF NOT EXISTS(SELECT FROM identity_access.session_login_bindings b
        JOIN identity_access.flow_source_bindings f ON f.flow_fingerprint=b.flow_fingerprint
        WHERE b.session_id=NEW.session_id AND f.membership_version=NEW.membership_version
            AND NEW.policy_version>=b.policy_version AND NEW.policy_version>=f.policy_version AND NEW.rights_version>=f.rights_version
            AND (NEW.policy_version<>f.policy_version OR (NEW.raw_idle,NEW.raw_absolute,NEW.raw_step_up)=(f.raw_idle,f.raw_absolute,f.raw_step_up))
            AND (NEW.rights_version<>f.rights_version OR NEW.raw_mfa_required=f.raw_mfa_required)) THEN
        RAISE EXCEPTION 'invalid session source binding' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION identity_access.require_source_binding() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$ BEGIN
    IF TG_TABLE_NAME='login_flows' THEN
        IF EXISTS(SELECT FROM identity_access.login_flows f WHERE f.fingerprint=NEW.fingerprint)
            AND NOT EXISTS(SELECT FROM identity_access.flow_source_bindings s WHERE s.flow_fingerprint=NEW.fingerprint) THEN
            RAISE EXCEPTION 'missing flow sources' USING ERRCODE='23514';
        END IF;
    ELSIF NOT EXISTS(SELECT FROM identity_access.session_source_observations s WHERE s.session_id=NEW.session_id) THEN
        RAISE EXCEPTION 'missing session sources' USING ERRCODE='23514';
    END IF;
    RETURN NULL;
END $$;
CREATE FUNCTION identity_access.guard_source_lineage() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$ BEGIN
    IF TG_TABLE_NAME='session_rotation_links' THEN
        IF NOT EXISTS(SELECT FROM identity_access.session_source_observations p
            JOIN identity_access.session_source_observations s ON s.session_id=NEW.successor
            WHERE p.session_id=NEW.predecessor AND
                (p.membership_version,p.policy_version,p.rights_version,p.raw_idle,p.raw_absolute,p.raw_step_up,p.raw_mfa_required)
                =(s.membership_version,s.policy_version,s.rights_version,s.raw_idle,s.raw_absolute,s.raw_step_up,s.raw_mfa_required)) THEN
            RAISE EXCEPTION 'invalid rotated sources' USING ERRCODE='23514';
        END IF;
    ELSIF NOT EXISTS(SELECT FROM identity_access.session_source_observations s
        JOIN identity_access.flow_source_bindings f ON f.flow_fingerprint=NEW.proof_fingerprint
        JOIN identity_access.flow_source_bindings o ON o.flow_fingerprint=NEW.origin_flow
        WHERE s.session_id=NEW.successor AND s.membership_version=f.membership_version AND s.membership_version=o.membership_version
            AND s.policy_version>=f.policy_version AND s.rights_version>=f.rights_version
            AND (s.policy_version<>f.policy_version OR (s.raw_idle,s.raw_absolute,s.raw_step_up)=(f.raw_idle,f.raw_absolute,f.raw_step_up))
            AND (s.rights_version<>f.rights_version OR s.raw_mfa_required=f.raw_mfa_required)) THEN
        RAISE EXCEPTION 'invalid proof sources' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_flow_sources(),identity_access.guard_session_sources(),identity_access.require_source_binding(),identity_access.guard_source_lineage() FROM PUBLIC;
CREATE TRIGGER flow_sources_guard BEFORE INSERT OR UPDATE ON identity_access.flow_source_bindings FOR EACH ROW EXECUTE FUNCTION identity_access.guard_flow_sources();
CREATE TRIGGER session_sources_guard BEFORE INSERT OR UPDATE ON identity_access.session_source_observations FOR EACH ROW EXECUTE FUNCTION identity_access.guard_session_sources();
CREATE CONSTRAINT TRIGGER complete_flow_sources AFTER INSERT ON identity_access.login_flows DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION identity_access.require_source_binding();
CREATE CONSTRAINT TRIGGER complete_session_sources AFTER INSERT ON identity_access.session_login_bindings DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION identity_access.require_source_binding();
CREATE TRIGGER rotation_sources_guard BEFORE INSERT ON identity_access.session_rotation_links FOR EACH ROW EXECUTE FUNCTION identity_access.guard_source_lineage();
CREATE TRIGGER proof_sources_guard BEFORE INSERT ON identity_access.step_up_receipts FOR EACH ROW EXECUTE FUNCTION identity_access.guard_source_lineage();
ALTER TABLE identity_access.flow_source_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.flow_source_bindings FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_source_observations ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_source_observations FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.flow_source_bindings,identity_access.session_source_observations FROM PUBLIC;
GRANT SELECT,INSERT ON identity_access.flow_source_bindings,identity_access.session_source_observations TO kc_identity_runtime;
GRANT UPDATE(membership_version,policy_version,rights_version,raw_idle,raw_absolute,raw_step_up,raw_mfa_required) ON identity_access.session_source_observations TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES(11);
