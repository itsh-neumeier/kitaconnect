import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createOrganizationFixtures, serializeOrganizationFixtures, validateOrganizationFixtures } from './fixtures.mjs';

test('Versionierte Testdaten sind bytegenau reproduzierbar und decken die Organisationsfälle ab', () => {
  const fixture = createOrganizationFixtures();
  assert.equal(readFileSync(new URL('../tests/fixtures/organizations.json', import.meta.url), 'utf8'), serializeOrganizationFixtures());
  assert.equal(fixture.carriers.length, 2);
  assert.equal(fixture.facilities.filter(f => f.carrier === null).length, 1);
  for (const carrier of fixture.carriers) assert.ok(fixture.facilities.some(f => f.carrier === carrier));
  assert.doesNotThrow(() => validateOrganizationFixtures(fixture));
});

const invalidCases = [
  ['Fehlende synthetische Kennzeichnung', f => { delete f.synthetic; }],
  ['Zusätzliche Personendatenfelder', f => { f.facilities[0].contact = 'synthetic-contact'; }],
  ['Unbekannte Metadatenfelder', f => { f.source = 'external'; }],
  ['Doppelte Trägerkennung', f => { f.carriers.push(f.carriers[0]); }],
  ['Doppelte Einrichtungskennung', f => { f.facilities.push({ ...f.facilities[0] }); }],
  ['Nicht registrierter Träger', f => { f.facilities[0].carrier = 'synthetic-carrier-missing'; }],
  ['Kennung ohne synthetischen Namensraum', f => { f.facilities[0].id = 'facility-a'; }],
  ['Ungültige Datentypen', f => { f.facilities[0] = null; }]
];
for (const [name, mutate] of invalidCases) {
  test(`${name}: Prüfung schlägt fehl`, () => {
    const fixture = createOrganizationFixtures();
    mutate(fixture);
    assert.throws(() => validateOrganizationFixtures(fixture));
  });
}
