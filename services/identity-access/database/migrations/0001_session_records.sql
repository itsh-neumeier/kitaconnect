-- Apply once with psql --single-transaction --set=ON_ERROR_STOP=1.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006, 1);
ALTER DEFAULT PRIVILEGES IN SCHEMA identity_access REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
ALTER DEFAULT PRIVILEGES IN SCHEMA identity_access REVOKE USAGE ON TYPES FROM PUBLIC;

CREATE DOMAIN identity_access.internal_id AS numeric
    CHECK (VALUE > 0 AND VALUE <= 340282366920938463463374607431768211455 AND VALUE = trunc(VALUE));
CREATE DOMAIN identity_access.epoch_second AS numeric
    CHECK (VALUE >= 0 AND VALUE <= 18446744073709551615 AND VALUE = trunc(VALUE));
CREATE DOMAIN identity_access.positive_seconds AS identity_access.epoch_second CHECK (VALUE > 0);
REVOKE ALL ON DOMAIN identity_access.internal_id, identity_access.epoch_second, identity_access.positive_seconds FROM PUBLIC;
GRANT USAGE ON DOMAIN identity_access.internal_id, identity_access.epoch_second, identity_access.positive_seconds TO kc_identity_runtime;

CREATE TABLE identity_access.identities (id identity_access.internal_id PRIMARY KEY);
CREATE TABLE identity_access.sessions (
    id identity_access.internal_id PRIMARY KEY,
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.identities(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    fingerprint bytea NOT NULL UNIQUE CHECK (octet_length(fingerprint) = 32),
    created_at identity_access.epoch_second NOT NULL,
    absolute_expires_at identity_access.epoch_second NOT NULL,
    idle_seconds identity_access.positive_seconds NOT NULL,
    last_activity_at identity_access.epoch_second NOT NULL,
    last_observed_at identity_access.epoch_second NOT NULL,
    state text COLLATE "C" NOT NULL CHECK (state IN ('active', 'expired', 'revoked')),
    CHECK (created_at <= last_activity_at AND last_activity_at <= last_observed_at AND last_activity_at < absolute_expires_at),
    CHECK (state <> 'active' OR last_observed_at < LEAST(absolute_expires_at, last_activity_at + idle_seconds))
);
CREATE INDEX sessions_identity_id ON identity_access.sessions(identity_id);

CREATE FUNCTION identity_access.guard_session_transition() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path = pg_catalog AS $$
BEGIN
    IF (NEW.id, NEW.identity_id, NEW.fingerprint, NEW.created_at, NEW.absolute_expires_at, NEW.idle_seconds)
        IS DISTINCT FROM (OLD.id, OLD.identity_id, OLD.fingerprint, OLD.created_at, OLD.absolute_expires_at, OLD.idle_seconds)
        OR NEW.last_observed_at < OLD.last_observed_at OR NEW.last_activity_at < OLD.last_activity_at
        OR (OLD.state = 'revoked' AND NEW.state <> 'revoked')
        OR (OLD.state = 'expired' AND NEW.state NOT IN ('expired', 'revoked'))
        OR (OLD.state <> 'active' AND NEW.last_activity_at <> OLD.last_activity_at) THEN
        RAISE EXCEPTION 'invalid session record transition' USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_session_transition() FROM PUBLIC;
CREATE TRIGGER session_transition_guard BEFORE UPDATE ON identity_access.sessions
    FOR EACH ROW EXECUTE FUNCTION identity_access.guard_session_transition();

ALTER TABLE identity_access.identities ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.identities FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.sessions FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.identities, identity_access.sessions FROM PUBLIC;
GRANT USAGE ON SCHEMA identity_access TO kc_identity_runtime;
GRANT SELECT, INSERT ON identity_access.identities, identity_access.sessions TO kc_identity_runtime;
GRANT UPDATE (last_activity_at, last_observed_at, state) ON identity_access.sessions TO kc_identity_runtime;
-- No permissive policy: application access remains closed until explicitly approved.

CREATE TABLE identity_access.schema_versions (version integer PRIMARY KEY CHECK (version > 0));
REVOKE ALL ON identity_access.schema_versions FROM PUBLIC;
INSERT INTO identity_access.schema_versions VALUES (1);
