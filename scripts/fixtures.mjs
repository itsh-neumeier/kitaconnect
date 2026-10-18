import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function createOrganizationFixtures() {
  return {
    synthetic: true,
    carriers: ['a', 'b'].map(id => `synthetic-carrier-${id}`),
    facilities: [
      ...['a', 'b'].map(id => ({ id: `synthetic-facility-${id}`, carrier: `synthetic-carrier-${id}` })),
      { id: 'synthetic-facility-independent', carrier: null }
    ]
  };
}

function exactKeys(value, keys) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    && Object.keys(value).length === keys.length
    && keys.every(key => Object.hasOwn(value, key));
}

export function validateOrganizationFixtures(value) {
  if (!exactKeys(value, ['synthetic', 'carriers', 'facilities']) || value.synthetic !== true
    || !Array.isArray(value.carriers) || !Array.isArray(value.facilities)) {
    throw new Error('Ungültige Testdatenstruktur oder fehlende Kennzeichnung');
  }
  if (value.carriers.some(id => typeof id !== 'string' || !/^synthetic-carrier-[a-z0-9-]+$/.test(id))
    || new Set(value.carriers).size !== value.carriers.length) {
    throw new Error('Ungültige oder doppelte Trägerkennung');
  }
  const facilities = new Set();
  for (const facility of value.facilities) {
    if (!exactKeys(facility, ['id', 'carrier']) || typeof facility.id !== 'string'
      || !/^synthetic-facility-[a-z0-9-]+$/.test(facility.id) || facilities.has(facility.id)) {
      throw new Error('Ungültige oder doppelte Einrichtungskennung');
    }
    if (facility.carrier !== null && !value.carriers.includes(facility.carrier)) {
      throw new Error('Unbekannte Trägerzuordnung');
    }
    facilities.add(facility.id);
  }
}

export function serializeOrganizationFixtures() {
  const fixture = createOrganizationFixtures();
  validateOrganizationFixtures(fixture);
  return JSON.stringify(fixture, null, 2) + '\n';
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length > 1 || (args.length === 1 && args[0] !== '--check')) throw new Error('Zulässig: fixtures.mjs [--check]');
  const path = resolve(dirname(fileURLToPath(import.meta.url)), '../tests/fixtures/organizations.json');
  const expected = serializeOrganizationFixtures();
  if (args[0] === '--check') {
    if (readFileSync(path, 'utf8') !== expected) throw new Error('Testdaten weichen vom Generator ab. Führen Sie npm run fixtures:generate aus.');
    console.log('Synthetische Organisationsdaten stimmen bytegenau mit dem Generator überein.');
  } else {
    writeFileSync(path, expected, 'utf8');
    console.log('Synthetische Organisationsdaten erzeugt.');
  }
}
