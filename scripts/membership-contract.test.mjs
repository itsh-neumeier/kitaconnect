import { readFileSync } from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';

const source = JSON.parse(readFileSync(new URL('../contracts/openapi/organization-config/membership-v1.json', import.meta.url), 'utf8'));

// Focused security invariants, not a complete OpenAPI validator or a transport proof.
function validate(document) {
  function resolve(node) {
    if (!node?.$ref) return node;
    assert.ok(node.$ref.startsWith('#/'), 'Nur lokale Verweise');
    const target = node.$ref.slice(2).split('/').reduce((n, k) => n?.[k], document);
    assert.ok(target, 'Defekter Verweis');
    return resolve(target);
  }
  assert.equal(document.openapi, '3.1.1');
  assert.ok(!document.servers?.length, 'Keine produktive Servervorgabe');
  const operation = document.paths['/internal/v1/membership/read'].post;
  assert.deepEqual(operation.security ?? document.security, [{ serviceMTLS: [], serviceToken: [] }], 'mTLS und Token gemeinsam');
  assert.equal(document.components.securitySchemes.serviceMTLS.type, 'mutualTLS');
  assert.equal(document.components.securitySchemes.serviceToken.scheme, 'bearer');
  const request = operation.requestBody.content['application/json'].schema;
  assert.equal(request.additionalProperties, false);
  assert.deepEqual(request.required, ['identity_ref', 'facility_id']);
  for (const [status, value] of Object.entries(operation.responses)) {
    const response = resolve(value);
    assert.equal(resolve(response.headers['Cache-Control']).schema.const, 'no-store', 'Cache-Schutz');
    assert.equal(resolve(response.headers.Pragma).schema.const, 'no-cache', 'Cache-Schutz');
    assert.deepEqual(Object.keys(response.content), [status === '200' ? 'application/json' : 'application/problem+json'], 'Problem Details');
  }
  const membership = document.components.schemas.Membership;
  assert.equal(membership.additionalProperties, false);
  assert.ok(!('mfa_required' in membership.properties), 'Keine erfundene MFA-Entscheidung');
}

test('Mitgliedschaftsvertrag bindet beide Dienstnachweise und schützt alle Antworten', () => validate(source));

test('Mitgliedschaftsvertrag weist alternative oder fehlende Dienstnachweise ab', () => {
  for (const security of [[], [{ serviceMTLS: [] }], [{ serviceMTLS: [] }, { serviceToken: [] }]]) {
    const document = structuredClone(source);
    document.paths['/internal/v1/membership/read'].post.security = security;
    assert.throws(() => validate(document), /gemeinsam/);
  }
});

test('Mitgliedschaftsvertrag weist Cache-Freigabe und falsches Fehlerformat ab', () => {
  const cache = structuredClone(source);
  cache.components.headers.NoStore.schema.const = 'public';
  assert.throws(() => validate(cache), /Cache-Schutz/);
  const format = structuredClone(source);
  format.components.responses.Problem.content = { 'application/json': {} };
  assert.throws(() => validate(format), /Problem Details/);
});
