import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { validateMainRuleset } from './check-github-rules.mjs';

const plan = JSON.parse(readFileSync(new URL('../.github/rulesets/main.json', import.meta.url), 'utf8'));
test('main: vollständiger Plan und API-Antwort mit Metadaten zulässig', () => {
  validateMainRuleset(plan);
  validateMainRuleset({ ...plan, id: 123, source_type: 'Repository' });
});
const weakenings = {
  'inaktiver Schutz': value => { value.enforcement = 'evaluate'; },
  'Administratorausnahme': value => { value.bypass_actors = [{ actor_type: 'RepositoryRole', actor_id: 5, bypass_mode: 'always' }]; },
  'falscher Branch': value => { value.conditions.ref_name.include = ['refs/heads/develop']; },
  'ausgenommener Branch': value => { value.conditions.ref_name.exclude = ['refs/heads/main']; },
  'Löschen erlaubt': value => { value.rules = value.rules.filter(rule => rule.type !== 'deletion'); },
  'Force-Push erlaubt': value => { value.rules = value.rules.filter(rule => rule.type !== 'non_fast_forward'); },
  'kein Review': value => { value.rules[2].parameters.required_approving_review_count = 0; },
  'alter Review gültig': value => { value.rules[2].parameters.dismiss_stale_reviews_on_push = false; },
  'CODEOWNER optional': value => { value.rules[2].parameters.require_code_owner_review = false; },
  'letzter Push ungeprüft': value => { value.rules[2].parameters.require_last_push_approval = false; },
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
