-- Storage foundation only. Existing adapters reject the new purpose routes.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ DECLARE item record; changed integer:=0; BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6,7] THEN
        RAISE EXCEPTION 'step-up storage requires migration versions 1 to 7';
    END IF;
    -- Find the two existing route checks by their referenced column, not generated names.
    FOR item IN SELECT c.conname FROM pg_catalog.pg_constraint c
        JOIN pg_catalog.pg_attribute a ON a.attrelid=c.conrelid AND a.attname='route'
        WHERE c.conrelid='identity_access.login_flows'::regclass AND c.contype='c' AND a.attnum=ANY(c.conkey)
    LOOP
        EXECUTE pg_catalog.format('ALTER TABLE identity_access.login_flows DROP CONSTRAINT %I',item.conname);
        changed:=changed+1;
    END LOOP;
    IF changed<>2 THEN RAISE EXCEPTION 'unexpected route constraints'; END IF;
END $$;
ALTER TABLE identity_access.login_flows
    ADD CONSTRAINT login_flow_route CHECK (route IN ('password','passkey','reauth_password','reauth_passkey')),
    ADD CONSTRAINT login_flow_route_factors CHECK (
        (route IN ('password','reauth_password') AND NOT passkey_ok)
        OR (route IN ('passkey','reauth_passkey') AND NOT password_ok AND NOT totp_ok));
CREATE TABLE identity_access.step_up_targets (
    flow_fingerprint bytea PRIMARY KEY REFERENCES identity_access.login_flows(fingerprint),
    session_id identity_access.internal_id NOT NULL REFERENCES identity_access.sessions(id),
    session_fingerprint bytea NOT NULL CHECK (octet_length(session_fingerprint)=32),
    origin_flow bytea NOT NULL REFERENCES identity_access.login_flows(fingerprint)
);
CREATE TABLE identity_access.step_up_receipts (
    proof_fingerprint bytea PRIMARY KEY REFERENCES identity_access.step_up_targets(flow_fingerprint),
    successor identity_access.internal_id NOT NULL UNIQUE REFERENCES identity_access.sessions(id),
    origin_flow bytea NOT NULL REFERENCES identity_access.login_flows(fingerprint),
    authenticated_at identity_access.epoch_second NOT NULL
);
CREATE FUNCTION identity_access.guard_step_up_target() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM identity_access.login_flows f
        JOIN identity_access.sessions s ON s.id=NEW.session_id
        JOIN identity_access.session_login_bindings b ON b.session_id=s.id
        JOIN identity_access.session_policy_observations o ON o.session_id=s.id
        JOIN identity_access.login_flows origin ON origin.fingerprint=b.flow_fingerprint
        WHERE f.fingerprint=NEW.flow_fingerprint AND f.route IN ('reauth_password','reauth_passkey')
            AND NOT f.consumed AND NOT f.invalidated AND f.proof_digest IS NULL
            AND s.state='active' AND s.fingerprint=NEW.session_fingerprint
            AND f.identity_id=s.identity_id AND f.facility_id=b.facility_id
            AND NEW.origin_flow=b.flow_fingerprint AND origin.consumed AND NOT origin.invalidated
            AND origin.route IN ('password','passkey') AND origin.identity_id=s.identity_id
            AND origin.facility_id=b.facility_id AND f.created_at>=s.last_observed_at
            AND f.created_at<LEAST(s.absolute_expires_at,s.last_activity_at+s.idle_seconds,
                s.created_at+o.absolute_cap,s.last_activity_at+o.idle_cap)
    ) THEN RAISE EXCEPTION 'invalid step-up target' USING ERRCODE='23514'; END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION identity_access.guard_step_up_receipt() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM identity_access.step_up_targets t
        JOIN identity_access.login_flows f ON f.fingerprint=t.flow_fingerprint
        JOIN identity_access.sessions p ON p.id=t.session_id
        JOIN identity_access.sessions s ON s.id=NEW.successor
        JOIN identity_access.session_rotation_links link ON link.successor=s.id AND link.predecessor=p.id
        JOIN identity_access.session_login_bindings pb ON pb.session_id=p.id
        JOIN identity_access.session_login_bindings sb ON sb.session_id=s.id
        WHERE t.flow_fingerprint=NEW.proof_fingerprint AND NEW.origin_flow=t.origin_flow
            AND pb.flow_fingerprint=t.origin_flow AND sb.flow_fingerprint=t.origin_flow
            AND p.fingerprint=t.session_fingerprint AND p.state='revoked' AND s.state='active'
            AND f.route IN ('reauth_password','reauth_passkey') AND f.consumed AND NOT f.invalidated
            AND (f.passkey_ok OR (f.password_ok AND f.totp_ok)) AND f.proof_digest IS NOT NULL
            AND f.identity_id=p.identity_id AND f.identity_id=s.identity_id
            AND f.facility_id=pb.facility_id AND f.facility_id=sb.facility_id
            AND NEW.authenticated_at=f.proven_at AND NEW.authenticated_at>=pb.authenticated_at
            AND NEW.authenticated_at>=p.created_at AND NEW.authenticated_at<=s.last_observed_at
            AND f.last_observed<f.expires_at AND s.last_observed_at<f.expires_at
            AND s.last_observed_at>=f.last_observed
    ) THEN RAISE EXCEPTION 'invalid step-up receipt' USING ERRCODE='23514'; END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION identity_access.reject_step_up_mutation() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN RAISE EXCEPTION 'step-up records are immutable' USING ERRCODE='23514'; END $$;
REVOKE ALL ON FUNCTION identity_access.guard_step_up_target(),identity_access.guard_step_up_receipt(),identity_access.reject_step_up_mutation() FROM PUBLIC;
CREATE TRIGGER step_up_target_guard BEFORE INSERT ON identity_access.step_up_targets
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_step_up_target();
CREATE TRIGGER step_up_receipt_guard BEFORE INSERT ON identity_access.step_up_receipts
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_step_up_receipt();
CREATE TRIGGER step_up_target_immutable BEFORE UPDATE OR DELETE ON identity_access.step_up_targets
FOR EACH ROW EXECUTE FUNCTION identity_access.reject_step_up_mutation();
CREATE TRIGGER step_up_receipt_immutable BEFORE UPDATE OR DELETE ON identity_access.step_up_receipts
FOR EACH ROW EXECUTE FUNCTION identity_access.reject_step_up_mutation();
ALTER TABLE identity_access.step_up_targets ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.step_up_targets FORCE ROW LEVEL SECURITY;
ALTER TABLE identity_access.step_up_receipts ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.step_up_receipts FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.step_up_targets,identity_access.step_up_receipts FROM PUBLIC;
GRANT SELECT,INSERT ON identity_access.step_up_targets,identity_access.step_up_receipts TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES (8);
