-- Run once per database, as the separate provisioning identity.
CREATE SCHEMA organization_config AUTHORIZATION kc_org_owner;
REVOKE ALL ON SCHEMA organization_config FROM PUBLIC;
