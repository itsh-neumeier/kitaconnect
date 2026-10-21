-- Synthetic storage markers only. These are not cryptographic factor evidence.
CREATE OR REPLACE FUNCTION pg_temp.expect_state(command text, expected text) RETURNS void
LANGUAGE plpgsql AS $$
DECLARE actual text;
BEGIN
    BEGIN EXECUTE command;
    EXCEPTION WHEN OTHERS THEN
        GET STACKED DIAGNOSTICS actual=RETURNED_SQLSTATE;
    END;
    IF actual IS DISTINCT FROM expected THEN
        RAISE EXCEPTION 'expected SQLSTATE %, received %',expected,actual;
    END IF;
END $$;
DO $$
DECLARE
    n integer:=COALESCE((SELECT max(id)::integer FROM identity_access.identities),0)+10;
    origin bytea:=decode(lpad(to_hex(n),64,'0'),'hex');
    proof bytea:=decode(lpad(to_hex(n+1),64,'0'),'hex');
    pw_proof bytea:=decode(lpad(to_hex(n+2),64,'0'),'hex');
    next_proof bytea:=decode(lpad(to_hex(n+3),64,'0'),'hex');
    old_hash bytea:=decode(repeat('11',32),'hex');
    new_hash bytea:=decode(repeat('22',32),'hex');
    proof_hash bytea:=decode(repeat('33',32),'hex');
BEGIN
    -- Each invocation uses a new identity and a separate local login lineage.
    old_hash:=set_byte(old_hash,0,n); new_hash:=set_byte(new_hash,0,n);
    INSERT INTO identity_access.identities VALUES(n);
    INSERT INTO identity_access.login_flows
        (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,
         mfa_required,created_at,expires_at,last_observed,consumed,proof_digest,password_ok,proven_at)
        VALUES(origin,n,1,'password',1,900,1000,300,false,1,301,10,true,proof_hash,true,10);
    INSERT INTO identity_access.sessions VALUES(n,n,old_hash,20,1000,900,20,20,'active');
    INSERT INTO identity_access.session_login_bindings VALUES(n,origin,1,1,300,10,false);
    INSERT INTO identity_access.session_policy_observations VALUES(n,1,900,1000,300,900,980,300,false);
    INSERT INTO identity_access.login_flows
        (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,
         mfa_required,created_at,expires_at,last_observed)
        VALUES(proof,n,1,'reauth_passkey',1,900,1000,300,true,30,330,30),
              (pw_proof,n,1,'reauth_password',1,900,1000,300,true,30,330,30);
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_targets VALUES(%L,%s,%L,%L)',origin,n,old_hash,origin),'23514');
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_targets VALUES(%L,%s,%L,%L)',proof,n,new_hash,origin),'23514');
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_targets VALUES(%L,%s,%L,%L)',proof,n,old_hash,pw_proof),'23514');
    INSERT INTO identity_access.step_up_targets VALUES(proof,n,old_hash,origin),(pw_proof,n,old_hash,origin);
    PERFORM pg_temp.expect_state(format('UPDATE identity_access.step_up_targets SET session_id=%s WHERE flow_fingerprint=%L',n,proof),'23514');
    PERFORM pg_temp.expect_state(format('DELETE FROM identity_access.step_up_targets WHERE flow_fingerprint=%L',proof),'23514');
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_targets VALUES(%L,%s,%L,%L)',proof,n,old_hash,origin),'23505');
    -- Route is immutable, and each route still forbids the wrong factor combination.
    PERFORM pg_temp.expect_state(format('UPDATE identity_access.login_flows SET route=''password'' WHERE fingerprint=%L',proof),'23514');
    PERFORM pg_temp.expect_state(format('UPDATE identity_access.login_flows SET password_ok=true,proof_digest=%L,proven_at=40 WHERE fingerprint=%L',proof_hash,proof),'23514');
    PERFORM pg_temp.expect_state(format('UPDATE identity_access.login_flows SET passkey_ok=true,proof_digest=%L,proven_at=40 WHERE fingerprint=%L',proof_hash,pw_proof),'23514');
    UPDATE identity_access.sessions SET state='revoked',last_observed_at=50 WHERE id=n;
    INSERT INTO identity_access.sessions VALUES(n+1,n,new_hash,20,1000,900,20,50,'active');
    INSERT INTO identity_access.session_login_bindings VALUES(n+1,origin,1,1,300,10,false);
    INSERT INTO identity_access.session_policy_observations VALUES(n+1,1,900,1000,300,900,980,300,false);
    -- A receipt cannot be inserted before both proof consumption and the rotation ledger.
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,40)',proof,n+1,origin),'23514');
    INSERT INTO identity_access.session_rotation_links VALUES(n+1,n);
    UPDATE identity_access.login_flows SET passkey_ok=true,proof_digest=proof_hash,proven_at=40,last_observed=40 WHERE fingerprint=proof;
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,40)',proof,n+1,origin),'23514');
    UPDATE identity_access.login_flows SET consumed=true WHERE fingerprint=proof;
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,41)',proof,n+1,origin),'23514');
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,40)',proof,n+1,pw_proof),'23514');
    -- Password alone remains insufficient even if a synthetic fixture marks it consumed.
    UPDATE identity_access.login_flows SET password_ok=true,proof_digest=proof_hash,proven_at=40,last_observed=40,consumed=true WHERE fingerprint=pw_proof;
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,40)',pw_proof,n+1,origin),'23514');
    INSERT INTO identity_access.step_up_receipts VALUES(proof,n+1,origin,40);
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_receipts VALUES(%L,%s,%L,40)',proof,n+1,origin),'23505');
    PERFORM pg_temp.expect_state(format('UPDATE identity_access.step_up_receipts SET authenticated_at=41 WHERE successor=%s',n+1),'23514');
    PERFORM pg_temp.expect_state(format('DELETE FROM identity_access.step_up_receipts WHERE successor=%s',n+1),'23514');
    ASSERT (SELECT (created_at,absolute_expires_at,last_activity_at)=(20,1000,20) FROM identity_access.sessions WHERE id=n+1);
    ASSERT (SELECT (authenticated_at,mfa)=(10,false) FROM identity_access.session_login_bindings WHERE session_id=n+1);
    -- A subsequent synthetic password-plus-TOTP proof belongs to the current successor.
    INSERT INTO identity_access.login_flows
        (fingerprint,identity_id,facility_id,route,policy_version,idle_seconds,absolute_seconds,step_up_seconds,
         mfa_required,created_at,expires_at,last_observed)
        VALUES(next_proof,n,1,'reauth_password',1,900,1000,300,true,60,360,60);
    PERFORM pg_temp.expect_state(format('INSERT INTO identity_access.step_up_targets VALUES(%L,%s,%L,%L)',next_proof,n,old_hash,origin),'23514');
    INSERT INTO identity_access.step_up_targets VALUES(next_proof,n+1,new_hash,origin);
    UPDATE identity_access.login_flows SET password_ok=true,totp_ok=true,proof_digest=proof_hash,proven_at=70,last_observed=70,consumed=true WHERE fingerprint=next_proof;
    UPDATE identity_access.sessions SET state='revoked',last_observed_at=80 WHERE id=n+1;
    INSERT INTO identity_access.sessions VALUES(n+2,n,set_byte(decode(repeat('44',32),'hex'),0,n),20,1000,900,20,80,'active');
    INSERT INTO identity_access.session_login_bindings VALUES(n+2,origin,1,1,300,10,false);
    INSERT INTO identity_access.session_policy_observations VALUES(n+2,1,900,1000,300,900,980,300,false);
    INSERT INTO identity_access.session_rotation_links VALUES(n+2,n+1);
    INSERT INTO identity_access.step_up_receipts VALUES(next_proof,n+2,origin,70);
    ASSERT (SELECT (created_at,absolute_expires_at,last_activity_at)=(20,1000,20) FROM identity_access.sessions WHERE id=n+2);
END $$;
DO $$ BEGIN
    ASSERT (SELECT array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5,6,7,8] FROM identity_access.schema_versions);
    ASSERT (SELECT count(*)=2 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='identity_access' AND c.relname IN ('step_up_targets','step_up_receipts')
        AND c.relrowsecurity AND c.relforcerowsecurity AND pg_catalog.pg_get_userbyid(c.relowner)='kc_identity_owner');
    ASSERT NOT EXISTS(SELECT FROM pg_catalog.pg_policies WHERE schemaname='identity_access');
    ASSERT NOT pg_catalog.has_function_privilege('kc_identity_runtime','identity_access.guard_step_up_target()','EXECUTE');
    ASSERT NOT pg_catalog.has_function_privilege('kc_identity_runtime','identity_access.guard_step_up_receipt()','EXECUTE');
    ASSERT NOT pg_catalog.has_table_privilege('kc_identity_runtime','identity_access.step_up_receipts','UPDATE,DELETE,TRUNCATE');
END $$;
SET LOCAL ROLE kc_identity_runtime;
DO $$ BEGIN
    ASSERT (SELECT count(*)=0 FROM identity_access.step_up_targets);
    ASSERT (SELECT count(*)=0 FROM identity_access.step_up_receipts);
    BEGIN
        INSERT INTO identity_access.step_up_receipts VALUES(decode(repeat('00',32),'hex'),1,decode(repeat('00',32),'hex'),1);
        RAISE EXCEPTION 'runtime insert unexpectedly permitted';
    EXCEPTION WHEN insufficient_privilege THEN NULL; END;
END $$;
RESET ROLE;
-- Visibility is an explicit administrator fixture, never a runtime policy.
CREATE POLICY step_up_owner_fixture ON identity_access.step_up_targets TO kc_identity_owner USING(true) WITH CHECK(true);
CREATE POLICY step_up_owner_fixture ON identity_access.step_up_receipts TO kc_identity_owner USING(true) WITH CHECK(true);
SET LOCAL ROLE kc_identity_owner;
DO $$ BEGIN
    ASSERT (SELECT count(*)>0 FROM identity_access.step_up_targets);
    ASSERT (SELECT count(*)>0 FROM identity_access.step_up_receipts);
    PERFORM pg_temp.expect_state('UPDATE identity_access.step_up_targets SET session_id=session_id','23514');
    PERFORM pg_temp.expect_state('DELETE FROM identity_access.step_up_targets','23514');
    PERFORM pg_temp.expect_state('UPDATE identity_access.step_up_receipts SET authenticated_at=authenticated_at','23514');
    PERFORM pg_temp.expect_state('DELETE FROM identity_access.step_up_receipts','23514');
END $$;
RESET ROLE;
DROP POLICY step_up_owner_fixture ON identity_access.step_up_targets;
DROP POLICY step_up_owner_fixture ON identity_access.step_up_receipts;
DO $$ BEGIN ASSERT NOT EXISTS(SELECT FROM pg_catalog.pg_policies WHERE schemaname='identity_access'); END $$;
SET LOCAL ROLE kc_identity_owner;
DO $$ BEGIN
    ASSERT (SELECT count(*)=0 FROM identity_access.step_up_targets);
    ASSERT (SELECT count(*)=0 FROM identity_access.step_up_receipts);
END $$;
RESET ROLE;
