DO $$
BEGIN
    ASSERT NOT EXISTS (
        SELECT FROM pg_catalog.pg_roles WHERE rolname IN ('kc_org_owner', 'kc_org_migrator', 'kc_org_runtime')
        AND (rolcanlogin OR rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls OR rolinherit)
    );
    ASSERT NOT pg_catalog.pg_has_role('kc_org_runtime', 'kc_org_owner', 'MEMBER');
    ASSERT NOT pg_catalog.pg_has_role('kc_org_runtime', 'kc_org_migrator', 'MEMBER');
    ASSERT NOT pg_catalog.has_schema_privilege('kc_org_runtime', 'organization_config', 'CREATE');
    ASSERT NOT pg_catalog.has_table_privilege('kc_org_runtime', 'organization_config.schema_versions', 'SELECT');
    ASSERT (SELECT count(*) = 2 FROM pg_catalog.pg_class c
            JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'organization_config' AND c.relname IN ('carriers', 'facilities')
              AND c.relrowsecurity AND c.relforcerowsecurity
              AND c.relowner = 'kc_org_owner'::regrole);
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_policies WHERE schemaname = 'organization_config');
    ASSERT (SELECT count(*) = 1 FROM pg_catalog.pg_auth_members
            WHERE roleid = 'kc_org_owner'::regrole AND member = 'kc_org_migrator'::regrole
              AND set_option AND NOT inherit_option AND NOT admin_option);
END $$;

-- SESSION AUTHORIZATION is essential: SET ROLE alone would retain the test superuser's ability to switch roles.
SET SESSION AUTHORIZATION kc_org_runtime;
DO $$
BEGIN
    ASSERT (SELECT count(*) = 0 FROM organization_config.carriers);
    ASSERT (SELECT count(*) = 0 FROM organization_config.facilities);
    PERFORM pg_catalog.set_config('app.facility_id', '1', false);
    ASSERT (SELECT count(*) = 0 FROM organization_config.facilities);
    BEGIN
        INSERT INTO organization_config.facilities VALUES (5, NULL);
        RAISE EXCEPTION 'runtime insert bypassed RLS';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        UPDATE organization_config.facilities SET carrier_id = 2 WHERE id = 1;
        RAISE EXCEPTION 'runtime changed carrier';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        DELETE FROM organization_config.facilities;
        RAISE EXCEPTION 'runtime deleted data';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        TRUNCATE organization_config.facilities;
        RAISE EXCEPTION 'runtime truncated data';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        CREATE TABLE organization_config.shadow (id integer);
        RAISE EXCEPTION 'runtime created schema object';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        ALTER TABLE organization_config.facilities DISABLE ROW LEVEL SECURITY;
        RAISE EXCEPTION 'runtime disabled RLS';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        SET ROLE kc_org_owner;
        RAISE EXCEPTION 'runtime acquired owner';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
    BEGIN
        SET ROLE kc_org_migrator;
        RAISE EXCEPTION 'runtime acquired migration role';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
END $$;
RESET SESSION AUTHORIZATION;

SET SESSION AUTHORIZATION kc_org_owner;
DO $$
BEGIN
    ASSERT (SELECT count(*) = 0 FROM organization_config.facilities), 'FORCE RLS did not constrain owner';
END $$;
RESET SESSION AUTHORIZATION;
