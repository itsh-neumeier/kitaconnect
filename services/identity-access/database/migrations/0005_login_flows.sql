-- Apply after 0001..0004 as migration role in a single transaction.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4] THEN
        RAISE EXCEPTION 'login flows require migration versions 1 to 4';
    END IF;
END $$;
CREATE TABLE identity_access.login_limits (
    id integer PRIMARY KEY CHECK (id=1),
    lifetime integer NOT NULL CHECK (lifetime BETWEEN 1 AND 300),
    pending bigint NOT NULL CHECK (pending BETWEEN 1 AND 4294967295),
    per_identity bigint NOT NULL CHECK (per_identity BETWEEN 1 AND pending),
    last_observed identity_access.epoch_second NOT NULL
);
CREATE TABLE identity_access.login_flows (
    fingerprint bytea PRIMARY KEY CHECK (octet_length(fingerprint)=32),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.identities(id),
    facility_id identity_access.internal_id NOT NULL,
    route text COLLATE "C" NOT NULL CHECK (route IN ('password','passkey')),
    policy_version numeric NOT NULL CHECK (policy_version=trunc(policy_version) AND policy_version BETWEEN 1 AND 18446744073709551615),
    idle_seconds numeric NOT NULL CHECK (idle_seconds=trunc(idle_seconds) AND idle_seconds BETWEEN 1 AND 900),
    absolute_seconds numeric NOT NULL CHECK (absolute_seconds=trunc(absolute_seconds) AND absolute_seconds BETWEEN 1 AND 28800),
    step_up_seconds numeric NOT NULL CHECK (step_up_seconds=trunc(step_up_seconds) AND step_up_seconds BETWEEN 1 AND 300),
    mfa_required boolean NOT NULL,
    created_at identity_access.epoch_second NOT NULL,
    expires_at identity_access.epoch_second NOT NULL CHECK (expires_at>created_at AND expires_at-created_at<=300),
    last_observed identity_access.epoch_second NOT NULL CHECK (last_observed>=created_at),
    consumed boolean NOT NULL DEFAULT false,
    proof_digest bytea CHECK (octet_length(proof_digest)=32),
    password_ok boolean NOT NULL DEFAULT false,
    totp_ok boolean NOT NULL DEFAULT false,
    passkey_ok boolean NOT NULL DEFAULT false,
    proven_at identity_access.epoch_second,
    CHECK (idle_seconds<=absolute_seconds AND step_up_seconds<=absolute_seconds),
    CHECK (NOT totp_ok OR password_ok),
    CHECK ((route='password' AND NOT passkey_ok) OR (route='passkey' AND NOT password_ok AND NOT totp_ok)),
    CHECK ((password_ok OR passkey_ok) = (proof_digest IS NOT NULL)),
    CHECK ((proof_digest IS NULL) = (proven_at IS NULL)),
    CHECK (proven_at IS NULL OR (proven_at>=created_at AND proven_at<expires_at)),
    CHECK (NOT consumed OR proof_digest IS NOT NULL)
);
CREATE INDEX login_flows_pending ON identity_access.login_flows(identity_id,expires_at) WHERE NOT consumed;
CREATE TABLE identity_access.session_login_bindings (
    session_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.sessions(id),
    flow_fingerprint bytea NOT NULL UNIQUE REFERENCES identity_access.login_flows(fingerprint),
    facility_id identity_access.internal_id NOT NULL,
    policy_version numeric NOT NULL CHECK (policy_version=trunc(policy_version) AND policy_version BETWEEN 1 AND 18446744073709551615),
    step_up_seconds numeric NOT NULL CHECK (step_up_seconds=trunc(step_up_seconds) AND step_up_seconds BETWEEN 1 AND 300),
    authenticated_at identity_access.epoch_second NOT NULL,
    mfa boolean NOT NULL
);
CREATE FUNCTION identity_access.guard_login_flow() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF (NEW.fingerprint,NEW.identity_id,NEW.facility_id,NEW.route,NEW.policy_version,NEW.idle_seconds,NEW.absolute_seconds,
        NEW.step_up_seconds,NEW.mfa_required,NEW.created_at,NEW.expires_at)
        IS DISTINCT FROM (OLD.fingerprint,OLD.identity_id,OLD.facility_id,OLD.route,OLD.policy_version,OLD.idle_seconds,OLD.absolute_seconds,
        OLD.step_up_seconds,OLD.mfa_required,OLD.created_at,OLD.expires_at)
        OR NEW.last_observed<OLD.last_observed OR (OLD.consumed AND NEW IS DISTINCT FROM OLD)
        OR (OLD.password_ok AND NOT NEW.password_ok) OR (OLD.totp_ok AND NOT NEW.totp_ok) OR (OLD.passkey_ok AND NOT NEW.passkey_ok)
        OR (OLD.proof_digest IS NOT NULL AND NEW.proof_digest IS DISTINCT FROM OLD.proof_digest)
        OR (OLD.proven_at IS NOT NULL AND NEW.proven_at IS DISTINCT FROM OLD.proven_at) THEN
        RAISE EXCEPTION 'invalid login flow transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_login_flow() FROM PUBLIC;
CREATE TRIGGER login_flow_guard BEFORE UPDATE ON identity_access.login_flows FOR EACH ROW EXECUTE FUNCTION identity_access.guard_login_flow();
ALTER TABLE identity_access.login_flows ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.login_flows FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_login_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_login_bindings FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.login_limits ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.login_limits FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.login_flows,identity_access.session_login_bindings,identity_access.login_limits FROM PUBLIC;
GRANT SELECT,INSERT ON identity_access.login_flows,identity_access.session_login_bindings,identity_access.login_limits TO kc_identity_runtime;
GRANT UPDATE (last_observed) ON identity_access.login_limits TO kc_identity_runtime;
GRANT UPDATE (proof_digest,password_ok,totp_ok,passkey_ok,proven_at,last_observed,consumed),DELETE ON identity_access.login_flows TO kc_identity_runtime;
-- No permissive policies or cross-service foreign keys.
INSERT INTO identity_access.schema_versions VALUES (5);
