-- Executed from a new connection after a real server restart.
DO $$
BEGIN
    ASSERT (SELECT count(*) = 2 FROM organization_config.carriers);
    ASSERT (SELECT count(*) = 3 FROM organization_config.facilities);
    ASSERT (SELECT carrier_id = 1 FROM organization_config.facilities WHERE id = 1);
    ASSERT (SELECT carrier_id = 2 FROM organization_config.facilities WHERE id = 2);
    ASSERT (SELECT carrier_id IS NULL FROM organization_config.facilities WHERE id = 3);
    ASSERT NOT EXISTS (SELECT FROM organization_config.facilities WHERE id IN (4, 5, 9));
    ASSERT (SELECT array_agg(version) = ARRAY[1] FROM organization_config.schema_versions);
END $$;
