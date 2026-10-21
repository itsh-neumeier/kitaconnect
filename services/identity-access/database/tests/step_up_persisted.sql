-- Run after the same isolated server restarts, before adding another synthetic lineage.
DO $$ BEGIN
    ASSERT (SELECT count(*)=3 FROM identity_access.step_up_targets);
    ASSERT (SELECT count(*)=2 FROM identity_access.step_up_receipts);
    ASSERT EXISTS(SELECT FROM identity_access.step_up_receipts r
        JOIN identity_access.step_up_targets t ON t.flow_fingerprint=r.proof_fingerprint
        JOIN identity_access.session_rotation_links l ON l.successor=r.successor AND l.predecessor=t.session_id
        JOIN identity_access.sessions s ON s.id=r.successor
        WHERE r.authenticated_at=40 AND s.created_at=20 AND s.absolute_expires_at=1000);
END $$;
