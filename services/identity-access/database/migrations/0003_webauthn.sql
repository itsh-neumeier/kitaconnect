-- Apply after 0001 and 0002 as migration role in a single transaction.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006, 1);

CREATE TABLE identity_access.webauthn_accounts (
    identity_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.identities(id),
    user_handle uuid NOT NULL UNIQUE,
    revision bytea NOT NULL CHECK (octet_length(revision) = 32),
    last_observed identity_access.epoch_second NOT NULL
);
CREATE TABLE identity_access.webauthn_credentials (
    credential_id bytea PRIMARY KEY CHECK (octet_length(credential_id) BETWEEN 1 AND 1024),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.webauthn_accounts(identity_id),
    data text COLLATE "C" NOT NULL CHECK (octet_length(data) BETWEEN 1 AND 65536)
);
CREATE INDEX webauthn_credentials_identity ON identity_access.webauthn_credentials(identity_id);
CREATE TABLE identity_access.webauthn_ceremonies (
    fingerprint bytea PRIMARY KEY CHECK (octet_length(fingerprint) = 32),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.webauthn_accounts(identity_id),
    profile text COLLATE "C" NOT NULL CHECK (profile = 'localhost:3000/v1'),
    purpose text COLLATE "C" NOT NULL CHECK (purpose IN ('register', 'authenticate')),
    created_at identity_access.epoch_second NOT NULL,
    expires_at identity_access.epoch_second NOT NULL CHECK (expires_at > created_at),
    consumed boolean NOT NULL,
    expected_revision bytea NOT NULL CHECK (octet_length(expected_revision) = 32),
    state text COLLATE "C" NOT NULL CHECK (octet_length(state) BETWEEN 1 AND 65536)
);
CREATE INDEX webauthn_ceremonies_expiry ON identity_access.webauthn_ceremonies(expires_at);

ALTER TABLE identity_access.webauthn_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.webauthn_accounts FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.webauthn_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.webauthn_credentials FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.webauthn_ceremonies ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.webauthn_ceremonies FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.webauthn_accounts, identity_access.webauthn_credentials, identity_access.webauthn_ceremonies FROM PUBLIC;
GRANT SELECT, INSERT ON identity_access.webauthn_accounts, identity_access.webauthn_credentials, identity_access.webauthn_ceremonies TO kc_identity_runtime;
GRANT UPDATE (last_observed, revision) ON identity_access.webauthn_accounts TO kc_identity_runtime;
GRANT UPDATE (data), DELETE ON identity_access.webauthn_credentials TO kc_identity_runtime;
GRANT UPDATE (consumed), DELETE ON identity_access.webauthn_ceremonies TO kc_identity_runtime;
-- No permissive runtime policies and no production origin configuration.
INSERT INTO identity_access.schema_versions VALUES (3);
