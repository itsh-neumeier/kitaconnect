-- Apply as migration role within a single transaction, after migration 0001.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006, 1);

CREATE TABLE identity_access.password_credentials (
    identity_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.identities(id),
    phc text COLLATE "C" NOT NULL CHECK (octet_length(phc) BETWEEN 1 AND 256),
    revision bytea NOT NULL CHECK (octet_length(revision) = 32)
);
CREATE TABLE identity_access.password_budget (
    id integer PRIMARY KEY CHECK (id = 1),
    window_seconds bigint NOT NULL CHECK (window_seconds BETWEEN 1 AND 4294967295),
    per_key bigint NOT NULL CHECK (per_key BETWEEN 1 AND 4294967295),
    total_limit bigint NOT NULL CHECK (total_limit BETWEEN 1 AND 4294967295),
    key_limit bigint NOT NULL CHECK (key_limit BETWEEN 1 AND 4294967295),
    window_start identity_access.epoch_second NOT NULL,
    last_observed identity_access.epoch_second NOT NULL CHECK (last_observed >= window_start),
    used bigint NOT NULL CHECK (used >= 0 AND used <= total_limit)
);
CREATE TABLE identity_access.password_attempts (
    subject_key bytea PRIMARY KEY CHECK (octet_length(subject_key) = 32),
    window_start identity_access.epoch_second NOT NULL,
    used bigint NOT NULL CHECK (used BETWEEN 1 AND 4294967295)
);
CREATE INDEX password_attempts_window ON identity_access.password_attempts(window_start);

ALTER TABLE identity_access.password_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.password_credentials FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.password_budget ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.password_budget FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.password_attempts ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.password_attempts FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.password_credentials, identity_access.password_budget, identity_access.password_attempts FROM PUBLIC;
GRANT SELECT, INSERT ON identity_access.password_credentials, identity_access.password_budget, identity_access.password_attempts TO kc_identity_runtime;
GRANT UPDATE (phc, revision) ON identity_access.password_credentials TO kc_identity_runtime;
GRANT UPDATE (window_start, last_observed, used) ON identity_access.password_budget TO kc_identity_runtime;
GRANT UPDATE (used), DELETE ON identity_access.password_attempts TO kc_identity_runtime;
-- No permissive runtime policies, connection identities or client-selected role switches.
INSERT INTO identity_access.schema_versions VALUES (2);
