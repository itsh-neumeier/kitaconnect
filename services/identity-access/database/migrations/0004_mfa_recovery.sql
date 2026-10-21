-- Apply after 0001..0003 as migration role in one transaction.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path = pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006, 1);

CREATE TABLE identity_access.mfa_nonce_counters (
    key_tag bytea PRIMARY KEY CHECK (octet_length(key_tag) = 16),
    used bigint NOT NULL CHECK (used BETWEEN 1 AND 4294967295)
);
CREATE TABLE identity_access.mfa_accounts (
    identity_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.identities(id),
    last_observed identity_access.epoch_second NOT NULL,
    recovery_required boolean NOT NULL,
    generation bytea CHECK (octet_length(generation) = 16),
    permit_hash bytea CHECK (octet_length(permit_hash) = 32),
    permit_expires identity_access.epoch_second,
    CHECK ((permit_hash IS NULL) = (permit_expires IS NULL)),
    CHECK (recovery_required OR permit_hash IS NULL)
);
CREATE TABLE identity_access.totp_factors (
    factor_id bytea PRIMARY KEY CHECK (octet_length(factor_id) = 16),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.mfa_accounts(identity_id),
    state text COLLATE "C" NOT NULL CHECK (state IN ('pending', 'active')),
    key_tag bytea NOT NULL REFERENCES identity_access.mfa_nonce_counters(key_tag),
    nonce bytea NOT NULL CHECK (octet_length(nonce) = 12),
    ciphertext bytea NOT NULL CHECK (octet_length(ciphertext) = 36),
    created_at identity_access.epoch_second NOT NULL,
    expires_at identity_access.epoch_second NOT NULL CHECK (expires_at > created_at),
    last_step identity_access.epoch_second,
    UNIQUE (identity_id, state),
    UNIQUE (key_tag, nonce)
);
CREATE TABLE identity_access.mfa_recovery_codes (
    fingerprint bytea PRIMARY KEY CHECK (octet_length(fingerprint) = 32),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.mfa_accounts(identity_id),
    generation bytea NOT NULL CHECK (octet_length(generation) = 16),
    consumed boolean NOT NULL
);
CREATE INDEX mfa_recovery_identity ON identity_access.mfa_recovery_codes(identity_id);
-- Contact confirmation is a separate trusted procedure, never a caller-supplied address.
CREATE TABLE identity_access.mfa_notification_contacts (
    identity_id identity_access.internal_id PRIMARY KEY REFERENCES identity_access.identities(id),
    contact_ref uuid NOT NULL,
    confirmed_at identity_access.epoch_second NOT NULL
);
CREATE TABLE identity_access.mfa_security_events (
    id bytea PRIMARY KEY CHECK (octet_length(id) = 16),
    identity_id identity_access.internal_id NOT NULL REFERENCES identity_access.mfa_accounts(identity_id),
    kind text COLLATE "C" NOT NULL CHECK (kind IN ('enrollment', 'activation', 'recovery', 'revocation')),
    created_at identity_access.epoch_second NOT NULL
);
CREATE TABLE identity_access.mfa_notification_outbox (
    event_id bytea PRIMARY KEY REFERENCES identity_access.mfa_security_events(id),
    contact_ref uuid NOT NULL,
    state text COLLATE "C" NOT NULL DEFAULT 'pending' CHECK (state = 'pending')
);

ALTER TABLE identity_access.mfa_nonce_counters ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_nonce_counters FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_accounts FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.totp_factors ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.totp_factors FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_recovery_codes ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_recovery_codes FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_notification_contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_notification_contacts FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_security_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_security_events FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_notification_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.mfa_notification_outbox FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.mfa_nonce_counters, identity_access.mfa_accounts,
    identity_access.totp_factors, identity_access.mfa_recovery_codes,
    identity_access.mfa_notification_contacts, identity_access.mfa_security_events,
    identity_access.mfa_notification_outbox FROM PUBLIC;
GRANT SELECT, INSERT ON identity_access.mfa_nonce_counters, identity_access.mfa_accounts,
    identity_access.totp_factors, identity_access.mfa_recovery_codes,
    identity_access.mfa_security_events, identity_access.mfa_notification_outbox TO kc_identity_runtime;
GRANT SELECT ON identity_access.mfa_notification_contacts TO kc_identity_runtime;
GRANT UPDATE (used) ON identity_access.mfa_nonce_counters TO kc_identity_runtime;
GRANT UPDATE (last_observed, recovery_required, generation, permit_hash, permit_expires)
    ON identity_access.mfa_accounts TO kc_identity_runtime;
GRANT UPDATE (state, last_step), DELETE ON identity_access.totp_factors TO kc_identity_runtime;
GRANT UPDATE (consumed), DELETE ON identity_access.mfa_recovery_codes TO kc_identity_runtime;
-- No permissive runtime policies, counter deletion or notification dispatch.
INSERT INTO identity_access.schema_versions VALUES (4);
