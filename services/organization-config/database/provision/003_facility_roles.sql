-- Separate privileged maintenance bundles. Never assign them to an application identity.
CREATE ROLE kc_facility_schema_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
CREATE ROLE kc_facility_provisioner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
GRANT kc_facility_schema_owner TO kc_facility_provisioner WITH ADMIN FALSE, INHERIT FALSE, SET TRUE;
