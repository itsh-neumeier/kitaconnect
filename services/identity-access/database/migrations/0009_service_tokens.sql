-- Internal service-token storage only. Does not authenticate TLS connections.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6,7,8] THEN
        RAISE EXCEPTION 'service tokens require migration versions 1 to 8';
    END IF;
END $$;
CREATE TABLE identity_access.service_authority (
    singleton boolean PRIMARY KEY CHECK(singleton),
    installation bytea NOT NULL UNIQUE CHECK(octet_length(installation)=16),
    issuer text COLLATE "C" NOT NULL CHECK(length(issuer) BETWEEN 1 AND 2048)
);
CREATE TABLE identity_access.service_clients (
    name text COLLATE "C" PRIMARY KEY CHECK(name IN ('identity-access','organization-config','gateway')),
    version numeric NOT NULL CHECK(version=trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    active boolean NOT NULL DEFAULT false
);
CREATE TABLE identity_access.service_certificates (
    fingerprint bytea PRIMARY KEY CHECK(octet_length(fingerprint)=32),
    client text COLLATE "C" NOT NULL REFERENCES identity_access.service_clients(name),
    not_before identity_access.epoch_second NOT NULL,
    not_after identity_access.epoch_second NOT NULL CHECK(not_after>not_before AND not_after-not_before<=86400),
    revoked boolean NOT NULL DEFAULT true
);
CREATE TABLE identity_access.service_grants (
    client text COLLATE "C" NOT NULL REFERENCES identity_access.service_clients(name),
    target text COLLATE "C" NOT NULL REFERENCES identity_access.service_clients(name),
    scope text COLLATE "C" NOT NULL,
    version numeric NOT NULL CHECK(version=trunc(version) AND version BETWEEN 1 AND 18446744073709551615),
    active boolean NOT NULL DEFAULT false,
    PRIMARY KEY(client,target,scope),
    CHECK((client='identity-access' AND target='organization-config' AND scope='membership:read')
        OR(client='gateway' AND target='identity-access' AND scope IN ('login:local','session:check')))
);
CREATE TABLE identity_access.service_tokens (
    fingerprint bytea PRIMARY KEY CHECK(octet_length(fingerprint)=32),
    installation bytea NOT NULL REFERENCES identity_access.service_authority(installation),
    issuer text COLLATE "C" NOT NULL,
    client text COLLATE "C" NOT NULL,
    target text COLLATE "C" NOT NULL,
    scope text COLLATE "C" NOT NULL,
    certificate bytea NOT NULL REFERENCES identity_access.service_certificates(fingerprint),
    client_version numeric NOT NULL,
    target_version numeric NOT NULL,
    grant_version numeric NOT NULL,
    created_at identity_access.epoch_second NOT NULL,
    expires_at identity_access.epoch_second NOT NULL CHECK(expires_at>created_at AND expires_at-created_at<=300),
    last_observed identity_access.epoch_second NOT NULL CHECK(last_observed>=created_at),
    state text COLLATE "C" NOT NULL CHECK(state IN ('active','expired','revoked')),
    FOREIGN KEY(client,target,scope) REFERENCES identity_access.service_grants(client,target,scope),
    CHECK(state<>'active' OR last_observed<expires_at)
);
CREATE FUNCTION identity_access.guard_service_registry() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF TG_TABLE_NAME='service_authority' THEN
        RAISE EXCEPTION 'service authority is immutable' USING ERRCODE='23514';
    ELSIF TG_TABLE_NAME='service_certificates' THEN
        IF TG_OP='DELETE' OR (NEW.fingerprint,NEW.client,NEW.not_before,NEW.not_after)
            IS DISTINCT FROM (OLD.fingerprint,OLD.client,OLD.not_before,OLD.not_after)
            OR (OLD.revoked AND NOT NEW.revoked) THEN
            RAISE EXCEPTION 'invalid service certificate transition' USING ERRCODE='23514';
        END IF;
    ELSIF TG_OP='INSERT' THEN
        IF NEW.version<>1 THEN RAISE EXCEPTION 'invalid service registry version' USING ERRCODE='23514'; END IF;
    ELSIF TG_OP='DELETE' THEN
        RAISE EXCEPTION 'service registry deletion is not a revocation' USING ERRCODE='23514';
    ELSE
        IF NEW.version<>OLD.version+1 OR (to_jsonb(NEW)-'version'-'active') IS DISTINCT FROM (to_jsonb(OLD)-'version'-'active') THEN
            RAISE EXCEPTION 'invalid service registry transition' USING ERRCODE='23514';
        END IF;
    END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION identity_access.guard_service_token() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF TG_OP='INSERT' THEN
        IF NEW.state<>'active' OR NEW.last_observed<>NEW.created_at OR NOT EXISTS(
            SELECT FROM identity_access.service_authority a
            JOIN identity_access.service_clients c ON c.name=NEW.client
            JOIN identity_access.service_clients t ON t.name=NEW.target
            JOIN identity_access.service_certificates cert ON cert.fingerprint=NEW.certificate AND cert.client=c.name
            JOIN identity_access.service_grants g ON (g.client,g.target,g.scope)=(NEW.client,NEW.target,NEW.scope)
            WHERE a.installation=NEW.installation AND a.issuer=NEW.issuer AND c.active AND t.active AND g.active AND NOT cert.revoked
                AND (c.version,t.version,g.version)=(NEW.client_version,NEW.target_version,NEW.grant_version)
                AND NEW.created_at>=cert.not_before AND NEW.expires_at<=cert.not_after
        ) THEN RAISE EXCEPTION 'invalid service token binding' USING ERRCODE='23514'; END IF;
    ELSIF (to_jsonb(NEW)-'last_observed'-'state') IS DISTINCT FROM (to_jsonb(OLD)-'last_observed'-'state')
        OR NEW.last_observed<OLD.last_observed OR (OLD.state<>'active' AND NEW.state<>OLD.state) THEN
        RAISE EXCEPTION 'invalid service token transition' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_service_registry(),identity_access.guard_service_token() FROM PUBLIC;
CREATE TRIGGER service_authority_guard BEFORE UPDATE OR DELETE ON identity_access.service_authority FOR EACH ROW EXECUTE FUNCTION identity_access.guard_service_registry();
CREATE TRIGGER service_client_guard BEFORE INSERT OR UPDATE OR DELETE ON identity_access.service_clients FOR EACH ROW EXECUTE FUNCTION identity_access.guard_service_registry();
CREATE TRIGGER service_grant_guard BEFORE INSERT OR UPDATE OR DELETE ON identity_access.service_grants FOR EACH ROW EXECUTE FUNCTION identity_access.guard_service_registry();
CREATE TRIGGER service_certificate_guard BEFORE UPDATE OR DELETE ON identity_access.service_certificates FOR EACH ROW EXECUTE FUNCTION identity_access.guard_service_registry();
CREATE TRIGGER service_token_guard BEFORE INSERT OR UPDATE ON identity_access.service_tokens FOR EACH ROW EXECUTE FUNCTION identity_access.guard_service_token();
ALTER TABLE identity_access.service_authority ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_authority FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_clients FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_certificates ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_certificates FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_grants FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.service_tokens FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.service_authority,identity_access.service_clients,identity_access.service_certificates,identity_access.service_grants,identity_access.service_tokens FROM PUBLIC;
GRANT SELECT ON identity_access.service_authority,identity_access.service_clients,identity_access.service_certificates,identity_access.service_grants,identity_access.service_tokens TO kc_identity_runtime;
GRANT INSERT ON identity_access.service_tokens TO kc_identity_runtime;
GRANT UPDATE(last_observed,state) ON identity_access.service_tokens TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES(9);
