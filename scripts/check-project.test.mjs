import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { validateBacklog } from './check-project.mjs';

const item = (id, depends_on = []) => ({ id, phase: 'P0', depends_on, acceptance: ['Nachweis'], security_scope: 'Keine Echtdaten' });
test('Gemeinsame Voraussetzungen sind zulässig', () => {
  assert.doesNotThrow(() => validateBacklog([item('KC-001'), item('KC-002', ['KC-001']), item('KC-003', ['KC-001', 'KC-002'])]));
});
test('Fehlende Voraussetzung verhindert die Freigabe', () => {
  assert.throws(() => validateBacklog([item('KC-001', ['KC-999'])]), /Fehlende/);
});
test('Zyklen verhindern die Freigabe', () => {
  assert.throws(() => validateBacklog([item('KC-001', ['KC-002']), item('KC-002', ['KC-001'])]), /Zyklische/);
});
test('Doppelte Kennungen verhindern die Freigabe', () => {
  assert.throws(() => validateBacklog([item('KC-001'), item('KC-001')]), /Doppelte/);
});

test('Grundmodell kann eine Voraussetzung sein, während die Gesamtabnahme auf spätere Rechteprüfung wartet', () => {
  const parent = { ...item('KC-004'), work_packages: ['KC-004A', 'KC-004B'] };
  assert.doesNotThrow(() => validateBacklog([
    parent, item('KC-004A'), item('KC-005', ['KC-004A']), item('KC-004B', ['KC-005']),
  ]));
});
test('Gesamtabnahme darf keinen verdeckten Zyklus durch Teilpakete erzeugen', () => {
  assert.throws(() => validateBacklog([
    { ...item('KC-004'), work_packages: ['KC-004A'] },
    item('KC-004A', ['KC-005']), item('KC-005', ['KC-004']),
  ]), /Zyklische/);
});
test('Fehlende oder doppelte Teilpakete werden abgewiesen', () => {
  assert.throws(() => validateBacklog([{ ...item('KC-004'), work_packages: ['KC-004A'] }]), /Fehlendes Teilpaket/);
  assert.throws(() => validateBacklog([
    { ...item('KC-004'), work_packages: ['KC-004A', 'KC-004A'] }, item('KC-004A'),
  ]), /Ungültige Teilpakete/);
});
test('Nicht zugeordnete, fremde und phasenfremde Teilpakete werden abgewiesen', () => {
  assert.throws(() => validateBacklog([item('KC-004'), item('KC-004A')]), /Nicht zugeordnetes/);
  assert.throws(() => validateBacklog([
    { ...item('KC-004'), work_packages: ['KC-005'] }, item('KC-005'),
  ]), /Falsche Teilpaketzuordnung/);
  assert.throws(() => validateBacklog([
    { ...item('KC-004'), work_packages: ['KC-004A'] }, { ...item('KC-004A'), phase: 'P1' },
  ]), /Falsche Teilpaketzuordnung/);
});

test('Aktueller MFA-Vorlauf darf nicht von der Anmeldegesamtabnahme abhängen', () => {
  const backlog = JSON.parse(readFileSync(new URL('../backlog/refined-issues.json', import.meta.url), 'utf8'));
  assert.doesNotThrow(() => validateBacklog(backlog));
  backlog.find(item => item.id === 'KC-008A').depends_on.push('KC-006');
  assert.throws(() => validateBacklog(backlog), /Zyklische/);
});
