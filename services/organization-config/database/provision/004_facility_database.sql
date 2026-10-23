-- The database comes from the current privileged maintenance connection, never a client header.
DO $$
BEGIN
    EXECUTE pg_catalog.format('GRANT CREATE ON DATABASE %I TO kc_facility_provisioner', pg_catalog.current_database());
END $$;
