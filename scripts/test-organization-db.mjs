// Starts and removes only its own ephemeral test container. Never targets an existing database.
import { spawnSync } from 'node:child_process';
import { readFileSync, mkdtempSync, chmodSync, rmSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { resolve, dirname, join, basename } from 'node:path';
import { tmpdir } from 'node:os';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sqlRoot = join(root, 'services/organization-config/database');
const identityRoot = join(root, 'services/identity-access/database');
const image = 'postgres:18.6-bookworm@sha256:afc7e2d441324c0388fa80c3d24f733b4194a4eb7f47dd8ee2b08eb1a24a647c';
let container;
let socketDirectory;

function docker(args, input) {
  const result = spawnSync('docker', args, { encoding: 'utf8', input, timeout: 180_000, maxBuffer: 8 * 1024 * 1024 });
  if (result.error) throw result.error;
  return result;
}

function requireSuccess(result, operation) {
  if (result.status !== 0) throw new Error(`${operation}: ${result.stderr || result.stdout}`);
  return result.stdout.trim();
}

function sql(text, { database = 'kc_org_test', transaction = false, expectedState } = {}) {
  const args = ['exec', '-i', container, 'psql', '-X', '--set=ON_ERROR_STOP=1', '--set=VERBOSITY=verbose', '-U', 'postgres', '-d', database];
  if (transaction) args.push('--single-transaction');
  const result = docker(args, text);
  if (expectedState) {
    if (result.status === 0 || !result.stderr.includes(` ${expectedState}:`)) {
      throw new Error(`Expected SQLSTATE ${expectedState}, received: ${result.stderr || result.stdout}`);
    }
  } else {
    requireSuccess(result, 'PostgreSQL test');
  }
}

function file(path, directory = sqlRoot) {
  return readFileSync(join(directory, path), 'utf8');
}

function migrate(database, suffix = '', expectedState) {
  sql(`SET SESSION AUTHORIZATION kc_org_migrator;\n${file('migrations/0001_organization_references.sql')}\n${suffix}`, {
    database, transaction: true, expectedState,
  });
}

async function waitUntilReady() {
  for (let attempt = 0; attempt < 100; attempt++) {
    const process = docker(['exec', container, 'cat', '/proc/1/comm']);
    if (process.status === 0 && process.stdout.trim() === 'postgres') {
      const ready = docker(['exec', container, 'pg_isready', '-U', 'postgres', '-d', 'kc_org_test']);
      if (ready.status === 0) return;
    }
    await new Promise(resolve => setTimeout(resolve, 300));
  }
  throw new Error('PostgreSQL did not become ready');
}

function testAdapter(phase, target = 'postgres', test = 'storage_adapter_controls_references', packageName = 'kitaconnect-organization-config') {
  const result = spawnSync('cargo', [
    'test', '--locked', '--package', packageName, '--test', target,
    '--', '--ignored', '--exact', test, '--nocapture',
  ], {
    cwd: root, encoding: 'utf8', timeout: 180_000, maxBuffer: 8 * 1024 * 1024,
    env: { ...process.env, KC_ORG_TEST_SOCKET: socketDirectory, KC_ORG_TEST_PHASE: phase },
  });
  if (result.error || result.status !== 0) {
    console.error(result.stdout || 'No test output before process timeout');
    console.error(result.stderr || '');
    if (result.error) throw result.error;
  }
  requireSuccess(result, `Rust adapter test (${phase})`);
  if (!/test result: ok\. 1 passed;/.test(result.stdout)) throw new Error('Rust adapter test was not executed');
  console.log(`PASS: Rust/SQLx ${packageName}/${target} (${phase} restart)`);
}

try {
  if (process.platform !== 'linux') throw new Error('The isolated socket fixture requires Linux, Docker and Rust');
  socketDirectory = mkdtempSync(join(tmpdir(), 'kc-org-test-socket-'));
  chmodSync(socketDirectory, 0o777); // The container's PostgreSQL UID must create its socket here.
  container = requireSuccess(docker([
    'create', '--name', `kc-org-test-${randomUUID()}`, '--network', 'none',
    '--label', 'kitaconnect.purpose=organization-db-test',
    '--mount', `type=bind,src=${socketDirectory},dst=/var/run/postgresql`,
    '-e', 'POSTGRES_HOST_AUTH_METHOD=trust', '-e', 'POSTGRES_DB=kc_org_test', image,
  ]), 'Create isolated test container');
  if (!/^[a-f0-9]{64}$/.test(container)) throw new Error('Unexpected test container identifier');
  requireSuccess(docker(['start', container]), 'Start test database');
  await waitUntilReady();
  sql("DO $$ BEGIN ASSERT current_setting('server_version') LIKE '18.6%'; END $$;");

  // Run the new HTTP boundary first so failures do not wait for unrelated hashing tests.
  sql(file('provision/001_roles.sql', identityRoot), { transaction: true });
  sql('CREATE DATABASE kc_service_http_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_service_http_test', transaction: true });
  for (const name of [
    '0001_session_records.sql', '0002_password_checks.sql', '0003_webauthn.sql',
    '0004_mfa_recovery.sql', '0005_login_flows.sql', '0006_session_policy_observations.sql',
    '0007_session_rotation.sql', '0008_step_up_storage.sql', '0009_service_tokens.sql',
  ]) {
    sql(`SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/' + name, identityRoot)}`, { database: 'kc_service_http_test', transaction: true });
  }
  sql('CREATE ROLE kc_service_http_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_service_http_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_service_http_test', transaction: true });
  // Separate organizational database: the reader never receives identity database options.
  sql(file('provision/001_roles.sql'), { transaction: true });
  sql(file('provision/003_facility_roles.sql'), { transaction: true });
  sql('CREATE DATABASE kc_authenticated_membership_test;');
  sql(file('provision/002_schema.sql'), { database: 'kc_authenticated_membership_test', transaction: true });
  for (const name of ['0001_organization_references.sql', '0002_facility_schema_registry.sql', '0003_facility_session_policies.sql', '0004_memberships.sql']) {
    sql(`SET SESSION AUTHORIZATION kc_org_migrator;\n${file('migrations/' + name)}`, { database: 'kc_authenticated_membership_test', transaction: true });
  }
  sql('CREATE ROLE kc_authenticated_membership_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_org_runtime TO kc_authenticated_membership_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_authenticated_membership_test', transaction: true });
  testAdapter('before', 'service_http', 'actual_http_is_bound_current_and_bounded', 'kitaconnect-identity-access');

  sql(file('provision/002_schema.sql'), { transaction: true });

  sql('CREATE DATABASE kc_org_failed_migration;');
  sql(file('provision/002_schema.sql'), { database: 'kc_org_failed_migration', transaction: true });
  migrate('kc_org_failed_migration', 'SELECT 1 / 0;', '22012');
  sql(`DO $$ BEGIN
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'organization_config');
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_type t JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace WHERE n.nspname = 'organization_config');
    ASSERT NOT pg_catalog.has_schema_privilege('kc_org_runtime', 'organization_config', 'USAGE');
  END $$;`, { database: 'kc_org_failed_migration' });
  // The failed database can be migrated cleanly afterwards.
  migrate('kc_org_failed_migration');
  console.log('PASS: failed migration rolls back objects and grants; clean retry succeeds');

  migrate('kc_org_test');
  sql(file('tests/integrity.sql'));
  sql(file('tests/roles.sql'));
  console.log('PASS: ID boundaries, duplicate/reference constraints, role separation and default-deny RLS');

  // First statement must disappear when the second statement fails.
  sql('INSERT INTO organization_config.carriers VALUES (9); INSERT INTO organization_config.facilities VALUES (9, 999);', {
    transaction: true, expectedState: '23503',
  });
  sql('DO $$ BEGIN ASSERT NOT EXISTS (SELECT FROM organization_config.carriers WHERE id = 9); END $$;');
  migrate('kc_org_test', '', '42710');
  sql(file('tests/persisted.sql'));
  console.log('PASS: failed write and migration replay leave no partial changes');

  sql('CREATE DATABASE kc_org_adapter_test;');
  sql(file('provision/002_schema.sql'), { database: 'kc_org_adapter_test', transaction: true });
  migrate('kc_org_adapter_test');
  // This login exists only in this ephemeral fixture; the shipped provisioning remains NOLOGIN.
  sql('CREATE ROLE kc_org_adapter_runtime LOGIN INHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_org_runtime TO kc_org_adapter_runtime WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;');
  testAdapter('before');

  sql('CREATE DATABASE kc_facility_schema_test;');
  sql(file('provision/002_schema.sql'), { database: 'kc_facility_schema_test', transaction: true });
  sql(file('provision/004_facility_database.sql'), { database: 'kc_facility_schema_test', transaction: true });
  migrate('kc_facility_schema_test');
  sql('INSERT INTO organization_config.facilities (id) VALUES (1), (2), (3), (340282366920938463463374607431768211455);', { database: 'kc_facility_schema_test' });
  const registryMigration = `SET SESSION AUTHORIZATION kc_org_migrator;\n${file('migrations/0002_facility_schema_registry.sql')}`;
  sql(`${registryMigration}\nSELECT 1 / 0;`, { database: 'kc_facility_schema_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('organization_config.facility_schemas') IS NULL;
    ASSERT (SELECT count(*) = 4 FROM organization_config.facilities);
    ASSERT NOT EXISTS (SELECT 1 FROM pg_catalog.pg_policies WHERE schemaname = 'organization_config');
    ASSERT NOT pg_catalog.has_schema_privilege('kc_facility_provisioner', 'organization_config', 'USAGE');
    ASSERT NOT pg_catalog.has_column_privilege('kc_facility_provisioner', 'organization_config.facilities', 'id', 'SELECT');
    ASSERT (SELECT pg_catalog.array_agg(version) = ARRAY[1] FROM organization_config.schema_versions);
  END $$;`, { database: 'kc_facility_schema_test' });
  sql(registryMigration, { database: 'kc_facility_schema_test', transaction: true });
  sql('CREATE ROLE kc_facility_test_maintenance LOGIN INHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_facility_provisioner TO kc_facility_test_maintenance WITH INHERIT TRUE, SET TRUE, ADMIN FALSE;');
  testAdapter('before', 'provisioning', 'schema_provisioning_is_atomic_and_restricted');
  testAdapter('before', 'isolation', 'service_objects_enforce_roles_and_rls');
  testAdapter('before', 'pool', 'reused_pool_clears_context_and_observes_revocation');
  testAdapter('before', 'maintenance', 'concurrent_maintenance_preserves_registry_and_wave_versions');

  sql('CREATE DATABASE kc_identity_failed_migration;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_identity_failed_migration', transaction: true });
  const identityMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0001_session_records.sql', identityRoot)}`;
  sql(`${identityMigration}\nSELECT 1 / 0;`, { database: 'kc_identity_failed_migration', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'identity_access');
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'identity_access');
    ASSERT NOT EXISTS (SELECT FROM pg_catalog.pg_type t JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace WHERE n.nspname = 'identity_access');
    ASSERT NOT pg_catalog.has_schema_privilege('kc_identity_runtime', 'identity_access', 'USAGE');
  END $$;`, { database: 'kc_identity_failed_migration' });
  sql(identityMigration, { database: 'kc_identity_failed_migration', transaction: true });
  sql('CREATE DATABASE kc_identity_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_identity_test', transaction: true });
  sql(identityMigration, { database: 'kc_identity_test', transaction: true });
  sql(identityMigration, { database: 'kc_identity_test', transaction: true, expectedState: '42710' });
  sql('CREATE ROLE kc_identity_test_runtime LOGIN INHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_identity_test_runtime WITH ADMIN FALSE, INHERIT TRUE, SET FALSE;');
  testAdapter('before', 'postgres', 'current_sessions_are_atomic_and_fail_closed', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_password_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_password_test', transaction: true });
  sql(identityMigration, { database: 'kc_password_test', transaction: true });
  const passwordMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0002_password_checks.sql', identityRoot)}`;
  sql(`${passwordMigration}\nSELECT 1 / 0;`, { database: 'kc_password_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.password_credentials') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.password_budget') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.password_attempts') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version) = ARRAY[1] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_password_test' });
  sql(passwordMigration, { database: 'kc_password_test', transaction: true });
  sql(passwordMigration, { database: 'kc_password_test', transaction: true, expectedState: '42P07' });
  testAdapter('before', 'password', 'passwords_and_durable_attempts_fail_closed', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_webauthn_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_webauthn_test', transaction: true });
  sql(identityMigration, { database: 'kc_webauthn_test', transaction: true });
  sql(passwordMigration, { database: 'kc_webauthn_test', transaction: true });
  const webauthnMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0003_webauthn.sql', identityRoot)}`;
  sql(`${webauthnMigration}\nSELECT 1 / 0;`, { database: 'kc_webauthn_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.webauthn_accounts') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.webauthn_credentials') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.webauthn_ceremonies') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version) = ARRAY[1,2] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_webauthn_test' });
  sql(webauthnMigration, { database: 'kc_webauthn_test', transaction: true });
  sql(webauthnMigration, { database: 'kc_webauthn_test', transaction: true, expectedState: '42P07' });
  testAdapter('before', 'webauthn', 'webauthn_ceremonies_are_current_and_single_use', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_mfa_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_mfa_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration]) {
    sql(migration, { database: 'kc_mfa_test', transaction: true });
  }
  const mfaMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0004_mfa_recovery.sql', identityRoot)}`;
  sql(`${mfaMigration}\nSELECT 1 / 0;`, { database: 'kc_mfa_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.mfa_accounts') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.mfa_nonce_counters') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.totp_factors') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.mfa_recovery_codes') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.mfa_notification_contacts') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.mfa_security_events') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.mfa_notification_outbox') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_mfa_nonce()') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version) = ARRAY[1,2,3] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_mfa_test' });
  sql(mfaMigration, { database: 'kc_mfa_test', transaction: true });
  sql(mfaMigration, { database: 'kc_mfa_test', transaction: true, expectedState: '42P07' });
  testAdapter('before', 'mfa', 'totp_and_recovery_are_atomic_and_protected', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_session_policy_test;');
  sql(file('provision/002_schema.sql'), { database: 'kc_session_policy_test', transaction: true });
  migrate('kc_session_policy_test');
  sql(registryMigration, { database: 'kc_session_policy_test', transaction: true });
  const policyMigration = `SET SESSION AUTHORIZATION kc_org_migrator;\n${file('migrations/0003_facility_session_policies.sql')}`;
  sql(`${policyMigration}\nSELECT 1 / 0;`, { database: 'kc_session_policy_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('organization_config.facility_session_policies') IS NULL;
    ASSERT pg_catalog.to_regprocedure('organization_config.guard_session_policy()') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version) = ARRAY[1,2] FROM organization_config.schema_versions);
  END $$;`, { database: 'kc_session_policy_test' });
  sql(policyMigration, { database: 'kc_session_policy_test', transaction: true });
  sql(policyMigration, { database: 'kc_session_policy_test', transaction: true, expectedState: 'P0001' });
  testAdapter('before', 'session_policy', 'facility_session_policies_are_versioned_and_restricted');

  sql('CREATE DATABASE kc_login_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_login_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration]) {
    sql(migration, { database: 'kc_login_test', transaction: true });
  }
  const loginMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0005_login_flows.sql', identityRoot)}`;
  sql(`${loginMigration}\nSELECT 1 / 0;`, { database: 'kc_login_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.login_flows') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.login_limits') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.session_login_bindings') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_login_flow()') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version)=ARRAY[1,2,3,4] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_login_test' });
  sql(loginMigration, { database: 'kc_login_test', transaction: true });
  sql(loginMigration, { database: 'kc_login_test', transaction: true, expectedState: 'P0001' });
  testAdapter('before', 'login', 'login_completion_is_atomic_and_current', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_session_check_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_session_check_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration]) {
    sql(migration, { database: 'kc_session_check_test', transaction: true });
  }
  const observationMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0006_session_policy_observations.sql', identityRoot)}`;
  sql(`${observationMigration}\nSELECT 1 / 0;`, { database: 'kc_session_check_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.session_policy_observations') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_session_policy_observation()') IS NULL;
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_session_check_test' });
  sql(observationMigration, { database: 'kc_session_check_test', transaction: true });
  sql(observationMigration, { database: 'kc_session_check_test', transaction: true, expectedState: 'P0001' });
  sql('CREATE ROLE kc_session_check_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_session_check_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_session_check_test', transaction: true });
  testAdapter('before', 'session_check', 'session_checks_preserve_current_policy_and_logout', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_session_rotation_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_session_rotation_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration]) {
    sql(migration, { database: 'kc_session_rotation_test', transaction: true });
  }
  const rotationMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0007_session_rotation.sql', identityRoot)}`;
  sql(`${rotationMigration}\nSELECT 1 / 0;`, { database: 'kc_session_rotation_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.session_rotation_links') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_session_rotation()') IS NULL;
    ASSERT EXISTS(SELECT FROM pg_catalog.pg_constraint WHERE conrelid='identity_access.session_login_bindings'::regclass AND conname='session_login_bindings_flow_fingerprint_key');
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5,6] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_session_rotation_test' });
  sql(rotationMigration, { database: 'kc_session_rotation_test', transaction: true });
  sql(rotationMigration, { database: 'kc_session_rotation_test', transaction: true, expectedState: 'P0001' });
  sql('CREATE ROLE kc_session_rotation_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_session_rotation_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_session_rotation_test', transaction: true });
  testAdapter('before', 'session_rotation', 'rotation_preserves_login_and_logout_lineage', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_step_up_storage_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_step_up_storage_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration, rotationMigration]) {
    sql(migration, { database: 'kc_step_up_storage_test', transaction: true });
  }
  const stepUpMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0008_step_up_storage.sql', identityRoot)}`;
  sql(`${stepUpMigration}\nSELECT 1 / 0;`, { database: 'kc_step_up_storage_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('identity_access.step_up_targets') IS NULL;
    ASSERT pg_catalog.to_regclass('identity_access.step_up_receipts') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_step_up_target()') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.guard_step_up_receipt()') IS NULL;
    ASSERT pg_catalog.to_regprocedure('identity_access.reject_step_up_mutation()') IS NULL;
    ASSERT (SELECT count(*)=2 FROM pg_catalog.pg_constraint c JOIN pg_catalog.pg_attribute a
      ON a.attrelid=c.conrelid AND a.attname='route'
      WHERE c.conrelid='identity_access.login_flows'::regclass AND c.contype='c' AND a.attnum=ANY(c.conkey));
    ASSERT NOT EXISTS(SELECT FROM pg_catalog.pg_constraint WHERE conrelid='identity_access.login_flows'::regclass
      AND pg_catalog.pg_get_constraintdef(oid) LIKE '%reauth_%');
    ASSERT (SELECT pg_catalog.array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5,6,7] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_step_up_storage_test' });
  sql(stepUpMigration, { database: 'kc_step_up_storage_test', transaction: true });
  sql(stepUpMigration, { database: 'kc_step_up_storage_test', transaction: true, expectedState: 'P0001' });
  sql(file('tests/step_up_storage.sql', identityRoot), { database: 'kc_step_up_storage_test', transaction: true });
  console.log('PASS: synthetic step-up storage constraints and closed RLS (before restart)');

  sql('CREATE DATABASE kc_step_up_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_step_up_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration, rotationMigration, stepUpMigration]) {
    sql(migration, { database: 'kc_step_up_test', transaction: true });
  }
  sql('CREATE ROLE kc_step_up_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_step_up_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_step_up_test', transaction: true });
  testAdapter('before', 'step_up', 'additional_proof_is_bound_and_rotated', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_membership_test;');
  sql(file('provision/002_schema.sql'), { database: 'kc_membership_test', transaction: true });
  migrate('kc_membership_test');
  sql(registryMigration, { database: 'kc_membership_test', transaction: true });
  sql(policyMigration, { database: 'kc_membership_test', transaction: true });
  const membershipMigration = `SET SESSION AUTHORIZATION kc_org_migrator;\n${file('migrations/0004_memberships.sql')}`;
  sql(`${membershipMigration}\nSELECT 1 / 0;`, { database: 'kc_membership_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT pg_catalog.to_regclass('organization_config.memberships') IS NULL;
    ASSERT pg_catalog.to_regprocedure('organization_config.guard_membership()') IS NULL;
    ASSERT (SELECT array_agg(version ORDER BY version)=ARRAY[1,2,3] FROM organization_config.schema_versions);
  END $$;`, { database: 'kc_membership_test' });
  sql(membershipMigration, { database: 'kc_membership_test', transaction: true });
  sql(membershipMigration, { database: 'kc_membership_test', transaction: true, expectedState: 'P0001' });
  sql('CREATE ROLE kc_membership_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_org_runtime TO kc_membership_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_membership_test', transaction: true });
  testAdapter('before', 'membership', 'memberships_are_current_and_read_only');

  sql('CREATE DATABASE kc_service_token_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_service_token_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration, rotationMigration, stepUpMigration]) {
    sql(migration, { database: 'kc_service_token_test', transaction: true });
  }
  const serviceTokenMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0009_service_tokens.sql', identityRoot)}`;
  sql(`${serviceTokenMigration}\nSELECT 1 / 0;`, { database: 'kc_service_token_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT to_regclass('identity_access.service_tokens') IS NULL;
    ASSERT to_regclass('identity_access.service_authority') IS NULL;
    ASSERT to_regclass('identity_access.service_clients') IS NULL;
    ASSERT to_regclass('identity_access.service_certificates') IS NULL;
    ASSERT to_regclass('identity_access.service_grants') IS NULL;
    ASSERT (SELECT array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5,6,7,8] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_service_token_test' });
  sql(serviceTokenMigration, { database: 'kc_service_token_test', transaction: true });
  sql(serviceTokenMigration, { database: 'kc_service_token_test', transaction: true, expectedState: 'P0001' });
  sql('CREATE ROLE kc_service_token_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_service_token_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_service_token_test', transaction: true });
  testAdapter('before', 'service_tokens', 'service_tokens_are_current_and_bound', 'kitaconnect-identity-access');

  sql('CREATE DATABASE kc_service_auth_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_service_auth_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration, rotationMigration, stepUpMigration, serviceTokenMigration]) {
    sql(migration, { database: 'kc_service_auth_test', transaction: true });
  }
  sql('CREATE ROLE kc_service_auth_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_service_auth_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_service_auth_test', transaction: true });
  testAdapter('before', 'service_auth', 'actual_peer_authentication_is_current', 'kitaconnect-identity-access');


  sql('CREATE DATABASE kc_role_assurance_test;');
  sql(file('provision/002_schema.sql', identityRoot), { database: 'kc_role_assurance_test', transaction: true });
  for (const migration of [identityMigration, passwordMigration, webauthnMigration, mfaMigration, loginMigration, observationMigration, rotationMigration, stepUpMigration, serviceTokenMigration]) {
    sql(migration, { database: 'kc_role_assurance_test', transaction: true });
  }
  const roleAssuranceMigration = `SET SESSION AUTHORIZATION kc_identity_migrator;\n${file('migrations/0010_role_assurance.sql', identityRoot)}`;
  sql(`${roleAssuranceMigration}\nSELECT 1 / 0;`, { database: 'kc_role_assurance_test', transaction: true, expectedState: '22012' });
  sql(`DO $$ BEGIN
    ASSERT to_regclass('identity_access.role_assurance_scopes') IS NULL;
    ASSERT to_regprocedure('identity_access.guard_role_assurance()') IS NULL;
    ASSERT (SELECT array_agg(version ORDER BY version)=ARRAY[1,2,3,4,5,6,7,8,9] FROM identity_access.schema_versions);
  END $$;`, { database: 'kc_role_assurance_test' });
  sql(roleAssuranceMigration, { database: 'kc_role_assurance_test', transaction: true });
  sql(roleAssuranceMigration, { database: 'kc_role_assurance_test', transaction: true, expectedState: 'P0001' });
  sql('CREATE ROLE kc_role_assurance_runtime LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS; GRANT kc_identity_runtime TO kc_role_assurance_runtime WITH INHERIT TRUE, SET FALSE;', { database: 'kc_role_assurance_test', transaction: true });
  testAdapter('before', 'rights_assurance', 'current_roles_are_complete_versioned_and_restricted', 'kitaconnect-identity-access');

  requireSuccess(docker(['restart', container]), 'Restart same test database');
  await waitUntilReady();
  sql(file('tests/persisted.sql'));
  sql(file('tests/roles.sql'));
  testAdapter('after');
  testAdapter('after', 'provisioning', 'schema_provisioning_is_atomic_and_restricted');
  testAdapter('after', 'isolation', 'service_objects_enforce_roles_and_rls');
  testAdapter('after', 'pool', 'reused_pool_clears_context_and_observes_revocation');
  testAdapter('after', 'maintenance', 'concurrent_maintenance_preserves_registry_and_wave_versions');
  testAdapter('after', 'postgres', 'current_sessions_are_atomic_and_fail_closed', 'kitaconnect-identity-access');
  testAdapter('after', 'password', 'passwords_and_durable_attempts_fail_closed', 'kitaconnect-identity-access');
  testAdapter('after', 'webauthn', 'webauthn_ceremonies_are_current_and_single_use', 'kitaconnect-identity-access');
  testAdapter('after', 'mfa', 'totp_and_recovery_are_atomic_and_protected', 'kitaconnect-identity-access');
  testAdapter('after', 'session_policy', 'facility_session_policies_are_versioned_and_restricted');
  testAdapter('after', 'login', 'login_completion_is_atomic_and_current', 'kitaconnect-identity-access');
  testAdapter('after', 'session_check', 'session_checks_preserve_current_policy_and_logout', 'kitaconnect-identity-access');
  testAdapter('after', 'session_rotation', 'rotation_preserves_login_and_logout_lineage', 'kitaconnect-identity-access');
  sql(file('tests/step_up_persisted.sql', identityRoot), { database: 'kc_step_up_storage_test', transaction: true });
  sql(file('tests/step_up_storage.sql', identityRoot), { database: 'kc_step_up_storage_test', transaction: true });
  console.log('PASS: synthetic step-up storage constraints and closed RLS (after restart)');
  testAdapter('after', 'step_up', 'additional_proof_is_bound_and_rotated', 'kitaconnect-identity-access');
  testAdapter('after', 'membership', 'memberships_are_current_and_read_only');
  testAdapter('after', 'service_tokens', 'service_tokens_are_current_and_bound', 'kitaconnect-identity-access');
  testAdapter('after', 'service_auth', 'actual_peer_authentication_is_current', 'kitaconnect-identity-access');
  testAdapter('after', 'service_http', 'actual_http_is_bound_current_and_bounded', 'kitaconnect-identity-access');
  testAdapter('after', 'rights_assurance', 'current_roles_are_complete_versioned_and_restricted', 'kitaconnect-identity-access');
  console.log('PASS: two carriers and an independent facility, version and role boundaries survive server restart');
} finally {
  if (container && /^[a-f0-9]{64}$/.test(container)) {
    // PostgreSQL changes this bind mount's ownership/mode during startup.
    // Restore the host owner as well: /tmp's sticky bit otherwise prevents removal of the directory itself.
    try {
      requireSuccess(docker(['exec', '--user', '0', container, 'chown', `${process.getuid()}:${process.getgid()}`, '/var/run/postgresql']), 'Restore own socket directory owner');
      requireSuccess(docker(['exec', '--user', '0', container, 'chmod', '0777', '/var/run/postgresql']), 'Prepare own socket directory cleanup');
    } finally {
      requireSuccess(docker(['rm', '--force', '--volumes', container]), 'Remove own test container and its volumes');
    }
  }
  if (socketDirectory) {
    const absolute = resolve(socketDirectory);
    if (dirname(absolute) !== resolve(tmpdir()) || !basename(absolute).startsWith('kc-org-test-socket-')) {
      throw new Error('Refusing cleanup outside own temporary socket directory');
    }
    rmSync(absolute, { recursive: true });
  }
}
