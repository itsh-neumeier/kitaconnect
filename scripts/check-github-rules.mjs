import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

function validateBranch(value) {
  assert.equal(value.target, 'branch');
  assert.equal(value.enforcement, 'active');
  assert.deepEqual(value.conditions.ref_name, { include: ['refs/heads/main'], exclude: [] });
}

export function validateMainRuleset(value) {
  validateBranch(value);
  assert.deepEqual(value.bypass_actors, [], 'Keine Ausnahme von Pflichtprüfungen zulässig.');
  const rules = new Map(value.rules.map(rule => [rule.type, rule]));
  assert.equal(rules.size, value.rules.length, 'Doppelte Regeltypen.');
  for (const type of ['deletion', 'non_fast_forward', 'pull_request', 'required_status_checks']) {
    assert.ok(rules.has(type), `Fehlende Regel: ${type}`);
  }
  const review = rules.get('pull_request').parameters;
  assert.equal(review.required_approving_review_count, 0, 'Reviewpflicht gehört in den gesonderten Regelsatz.');
  assert.equal(review.required_review_thread_resolution, true);
  for (const key of ['dismiss_stale_reviews_on_push', 'require_code_owner_review', 'require_last_push_approval']) {
    assert.equal(review[key], false, `Review-Ausnahme wird durch Basisregel verhindert: ${key}`);
  }
  const checks = rules.get('required_status_checks').parameters;
  assert.equal(checks.strict_required_status_checks_policy, true);
  assert.notEqual(checks.do_not_enforce_on_create, true);
  for (const name of ['structure', 'rust', 'web', 'secrets', 'dependencies']) {
    assert.equal(checks.required_status_checks.filter(check => check.context === name && check.integration_id === 15368).length, 1, `Fehlende oder nicht an GitHub Actions gebundene Prüfung: ${name}`);
  }
}

export function validateReviewRuleset(value) {
  validateBranch(value);
  assert.deepEqual(value.bypass_actors, [{ actor_id: 77058548, actor_type: 'User', bypass_mode: 'pull_request' }], 'Nur das bestätigte Konto darf Review über einen PR ausnehmen.');
  assert.equal(value.rules.length, 1);
  assert.equal(value.rules[0].type, 'pull_request', 'Die Ausnahme darf nur Review-Regeln betreffen.');
  const review = value.rules[0].parameters;
  assert.ok(Number.isInteger(review.required_approving_review_count) && review.required_approving_review_count >= 1);
  for (const key of ['dismiss_stale_reviews_on_push', 'require_code_owner_review', 'require_last_push_approval', 'required_review_thread_resolution']) {
    assert.equal(review[key], true, `Fehlende Review-Sperre: ${key}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const path = process.argv[2] ?? fileURLToPath(new URL('../.github/rulesets/main.json', import.meta.url));
  validateMainRuleset(JSON.parse(readFileSync(path, 'utf8')));
  const reviewPath = process.argv[3] ?? fileURLToPath(new URL('../.github/rulesets/main-reviews.json', import.meta.url));
  validateReviewRuleset(JSON.parse(readFileSync(reviewPath, 'utf8')));
  console.log('Beide Regelsätze erfüllen die P0-Schutzanforderungen. Dies ist kein Nachweis ihrer Aktivierung.');
}
