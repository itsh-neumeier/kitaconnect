// Starts and removes only its own ephemeral test container. Never targets an existing database.
import { spawnSync } from 'node:child_process';
import { readFileSync, mkdtempSync, chmodSync, rmSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { resolve, dirname, join, basename } from 'node:path';
import { tmpdir } from 'node:os';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sqlRoot = join(root, 'services/organization-config/database');
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

function file(path) {
  return readFileSync(join(sqlRoot, path), 'utf8');
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

function testAdapter(phase, target = 'postgres', test = 'storage_adapter_controls_references') {
  const result = spawnSync('cargo', [
    'test', '--locked', '--package', 'kitaconnect-organization-config', '--test', target,
    '--', '--ignored', '--exact', test, '--nocapture',
  ], {
    cwd: root, encoding: 'utf8', timeout: 180_000, maxBuffer: 8 * 1024 * 1024,
    env: { ...process.env, KC_ORG_TEST_SOCKET: socketDirectory, KC_ORG_TEST_PHASE: phase },
  });
  if (result.error) throw result.error;
  requireSuccess(result, `Rust adapter test (${phase})`);
  if (!/test result: ok\. 1 passed;/.test(result.stdout)) throw new Error('Rust adapter test was not executed');
  console.log(`PASS: Rust/SQLx ${target} (${phase} restart)`);
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
  sql(file('provision/001_roles.sql'), { transaction: true });
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

  sql(file('provision/003_facility_roles.sql'), { transaction: true });
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

  requireSuccess(docker(['restart', container]), 'Restart same test database');
  await waitUntilReady();
  sql(file('tests/persisted.sql'));
  sql(file('tests/roles.sql'));
  testAdapter('after');
  testAdapter('after', 'provisioning', 'schema_provisioning_is_atomic_and_restricted');
  testAdapter('after', 'isolation', 'service_objects_enforce_roles_and_rls');
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
