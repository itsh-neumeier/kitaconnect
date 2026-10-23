-- Apply after 0001..0006 as the migration role in one transaction.
SET LOCAL ROLE kc_identity_owner;
SET LOCAL search_path=pg_catalog;
SELECT pg_catalog.pg_advisory_xact_lock(184006,1);
DO $$ BEGIN
    IF (SELECT array_agg(version ORDER BY version) FROM identity_access.schema_versions) IS DISTINCT FROM ARRAY[1,2,3,4,5,6] THEN
        RAISE EXCEPTION 'session rotation requires migration versions 1 to 6';
    END IF;
END $$;
ALTER TABLE identity_access.session_login_bindings DROP CONSTRAINT session_login_bindings_flow_fingerprint_key;
CREATE INDEX session_login_bindings_flow ON identity_access.session_login_bindings(flow_fingerprint);
CREATE TABLE identity_access.session_rotation_links (
    successor identity_access.internal_id PRIMARY KEY REFERENCES identity_access.sessions(id),
    predecessor identity_access.internal_id NOT NULL UNIQUE REFERENCES identity_access.sessions(id),
    CHECK (successor<>predecessor)
);
CREATE FUNCTION identity_access.guard_session_rotation() RETURNS trigger
LANGUAGE plpgsql SECURITY INVOKER SET search_path=pg_catalog AS $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM identity_access.sessions p JOIN identity_access.sessions s ON s.id=NEW.successor
        JOIN identity_access.session_login_bindings pb ON pb.session_id=p.id
        JOIN identity_access.session_login_bindings sb ON sb.session_id=s.id
        JOIN identity_access.session_policy_observations po ON po.session_id=p.id
        JOIN identity_access.session_policy_observations so ON so.session_id=s.id
        WHERE p.id=NEW.predecessor AND p.state='revoked' AND s.state='active'
            AND (p.identity_id,p.created_at,p.absolute_expires_at,p.idle_seconds,p.last_activity_at)
                IS NOT DISTINCT FROM (s.identity_id,s.created_at,s.absolute_expires_at,s.idle_seconds,s.last_activity_at)
            AND s.last_observed_at>=p.last_observed_at AND s.fingerprint<>p.fingerprint
            AND (pb.flow_fingerprint,pb.facility_id,pb.policy_version,pb.step_up_seconds,pb.authenticated_at,pb.mfa)
                IS NOT DISTINCT FROM (sb.flow_fingerprint,sb.facility_id,sb.policy_version,sb.step_up_seconds,sb.authenticated_at,sb.mfa)
            AND so.version>=po.version AND so.idle_cap<=po.idle_cap AND so.absolute_cap<=po.absolute_cap
            AND so.step_up_cap<=po.step_up_cap AND (NOT po.mfa_required OR so.mfa_required)
    ) THEN
        RAISE EXCEPTION 'invalid session rotation' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;
REVOKE ALL ON FUNCTION identity_access.guard_session_rotation() FROM PUBLIC;
CREATE TRIGGER session_rotation_guard BEFORE INSERT ON identity_access.session_rotation_links
FOR EACH ROW EXECUTE FUNCTION identity_access.guard_session_rotation();
ALTER TABLE identity_access.session_rotation_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_access.session_rotation_links FORCE ROW LEVEL SECURITY;
REVOKE ALL ON identity_access.session_rotation_links FROM PUBLIC;
GRANT SELECT,INSERT ON identity_access.session_rotation_links TO kc_identity_runtime;
INSERT INTO identity_access.schema_versions VALUES (7);
