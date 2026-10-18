import { test } from 'node:test';
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
