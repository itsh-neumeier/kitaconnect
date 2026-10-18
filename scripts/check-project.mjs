import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export function validateBacklog(items) {
  const byId = new Map();
  for (const item of items) {
    if (!/^KC-\d{3}$/.test(item.id) || byId.has(item.id)) throw new Error(`Doppelte oder ungültige ID: ${item.id}`);
    if (!/^P[0-9]$/.test(item.phase) || !item.acceptance?.length || !item.security_scope) throw new Error(`Unvollständiges Issue: ${item.id}`);
    byId.set(item.id, item);
  }
  const done = new Set();
  const active = new Set();
  function visit(id) {
    if (!byId.has(id)) throw new Error(`Fehlende Abhängigkeit: ${id}`);
    if (active.has(id)) throw new Error(`Zyklische Abhängigkeit: ${id}`);
    if (done.has(id)) return;
    active.add(id);
    for (const dep of byId.get(id).depends_on) visit(dep);
    active.delete(id);
    done.add(id);
  }
  for (const id of byId.keys()) visit(id);
}

export function checkProject(root) {
  const backlog = JSON.parse(readFileSync(join(root, 'backlog/refined-issues.json'), 'utf8'));
  validateBacklog(backlog);
  const docs = [];
  function walk(path) {
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (entry.isDirectory()) walk(join(path, entry.name));
      else if (entry.name.endsWith('.md')) docs.push(join(path, entry.name));
    }
  }
  walk(join(root, 'docs'));
  docs.push(join(root, 'README.md'));
  for (const file of docs) {
    const text = readFileSync(file, 'utf8');
    for (const match of text.matchAll(/\[[^\]]*\]\(([^\s)]+)(?:\s+"[^"]*")?\)/g)) {
      const link = match[1];
      if (/^(https?:|mailto:|#)/.test(link)) continue;
      const target = resolve(dirname(file), decodeURIComponent(link.split('#')[0]));
      if (!existsSync(target)) throw new Error(`Defekter Verweis in ${file}: ${link}`);
    }
  }
  const fixture = JSON.parse(readFileSync(join(root, 'tests/fixtures/organizations.json'), 'utf8'));
  if (fixture.synthetic !== true || new Set(fixture.facilities.map(f => f.id)).size !== fixture.facilities.length) throw new Error('Ungültige Testdaten');
  for (const facility of fixture.facilities) {
    if (!facility.id.startsWith('synthetic-') || (facility.carrier !== null && !fixture.carriers.includes(facility.carrier))) throw new Error('Ungültige Organisationszuordnung');
  }
  console.log(`${backlog.length} Issues, ${docs.length} Dokumente und synthetische Testdaten geprüft.`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) checkProject(resolve(dirname(fileURLToPath(import.meta.url)), '..'));
