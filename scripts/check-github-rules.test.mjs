import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { validateMainRuleset, validateReviewRuleset } from './check-github-rules.mjs';

const plan = JSON.parse(readFileSync(new URL('../.github/rulesets/main.json', import.meta.url), 'utf8'));
const reviewPlan = JSON.parse(readFileSync(new URL('../.github/rulesets/main-reviews.json', import.meta.url), 'utf8'));
test('main: vollständiger Plan und API-Antwort mit Metadaten zulässig', () => {
  validateMainRuleset(plan);
  validateMainRuleset({ ...plan, id: 123, source_type: 'Repository' });
  validateReviewRuleset(reviewPlan);
  validateReviewRuleset({ ...reviewPlan, id: 456, source_type: 'Repository' });
});
const weakenings = {
  'inaktiver Schutz': value => { value.enforcement = 'evaluate'; },
  'Administratorausnahme': value => { value.bypass_actors = [{ actor_type: 'RepositoryRole', actor_id: 5, bypass_mode: 'always' }]; },
  'falscher Branch': value => { value.conditions.ref_name.include = ['refs/heads/develop']; },
  'ausgenommener Branch': value => { value.conditions.ref_name.exclude = ['refs/heads/main']; },
  'Löschen erlaubt': value => { value.rules = value.rules.filter(rule => rule.type !== 'deletion'); },
  'Force-Push erlaubt': value => { value.rules = value.rules.filter(rule => rule.type !== 'non_fast_forward'); },
  'kein PR-Zwang': value => { value.rules = value.rules.filter(rule => rule.type !== 'pull_request'); },
  'offene Gespräche': value => { value.rules[2].parameters.required_review_thread_resolution = false; },
  'veralteter Basisstand': value => { value.rules[3].parameters.strict_required_status_checks_policy = false; },
  'fehlender Sicherheitstest': value => { value.rules[3].parameters.required_status_checks.pop(); },
  'beliebiger Statusabsender': value => { delete value.rules[3].parameters.required_status_checks[0].integration_id; }
};
for (const [name, weaken] of Object.entries(weakenings)) {
  test(`main weist Abschwächung ab: ${name}`, () => {
    const value = structuredClone(plan);
    weaken(value);
    assert.throws(() => validateMainRuleset(value));
  });
}

const invalidReviews = {
  'fremdes Konto': value => { value.bypass_actors[0].actor_id = 123; },
  'zusätzliches Konto': value => { value.bypass_actors.push({ ...value.bypass_actors[0], actor_id: 123 }); },
  'pauschale Administratorrolle': value => { value.bypass_actors[0] = { actor_id: 5, actor_type: 'RepositoryRole', bypass_mode: 'pull_request' }; },
  'direkter Push als Ausnahme': value => { value.bypass_actors[0].bypass_mode = 'always'; },
  'unsichtbare Ausnahme': value => { value.bypass_actors[0].bypass_mode = 'exempt'; },
  'inaktiver Review-Schutz': value => { value.enforcement = 'disabled'; },
  'falscher Review-Zielbranch': value => { value.conditions.ref_name.include = ['refs/heads/develop']; },
  'fehlende Review-Regel': value => { value.rules = []; },
  'keine Reviewpflicht': value => { value.rules[0].parameters.required_approving_review_count = 0; },
  'veraltete Genehmigung': value => { value.rules[0].parameters.dismiss_stale_reviews_on_push = false; },
  'fehlende CODEOWNER-Prüfung': value => { value.rules[0].parameters.require_code_owner_review = false; },
  'fehlende Genehmigung des letzten Pushs': value => { value.rules[0].parameters.require_last_push_approval = false; },
  'offene Review-Gespräche': value => { value.rules[0].parameters.required_review_thread_resolution = false; },
  'Pflichtprüfungen im ausnehmbaren Satz': value => { value.rules.push(structuredClone(plan.rules[3])); }
};
for (const [name, weaken] of Object.entries(invalidReviews)) {
  test(`Review-Ausnahme weist unerlaubte Erweiterung ab: ${name}`, () => {
    const value = structuredClone(reviewPlan);
    weaken(value);
    assert.throws(() => validateReviewRuleset(value));
  });
}
test('Konto-Ausnahme darf nicht in den Basisregelsatz kopiert werden', () => {
  assert.throws(() => validateMainRuleset({ ...plan, bypass_actors: reviewPlan.bypass_actors }));
});
