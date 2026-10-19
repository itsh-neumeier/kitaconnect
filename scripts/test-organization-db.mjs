// Starts and removes only its own ephemeral test container. Never targets an existing database.
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { resolve, dirname, join } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sqlRoot = join(root, 'services/organization-config/database');
const image = 'postgres:18.6-bookworm@sha256:afc7e2d441324c0388fa80c3d24f733b4194a4eb7f47dd8ee2b08eb1a24a647c';
let container;

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

try {
  container = requireSuccess(docker([
    'create', '--name', `kc-org-test-${randomUUID()}`, '--network', 'none',
    '--label', 'kitaconnect.purpose=organization-db-test',
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

  requireSuccess(docker(['restart', container]), 'Restart same test database');
  await waitUntilReady();
  sql(file('tests/persisted.sql'));
  sql(file('tests/roles.sql'));
  console.log('PASS: two carriers and an independent facility, version and role boundaries survive server restart');
} finally {
  if (container && /^[a-f0-9]{64}$/.test(container)) {
    requireSuccess(docker(['rm', '--force', '--volumes', container]), 'Remove own test container and its volumes');
  }
}
