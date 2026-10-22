import { readFileSync } from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { validateServiceOAuth } from './service-oauth-contract.mjs';

const source = JSON.parse(readFileSync(new URL('../contracts/openapi/identity-access/service-oauth-v1.json', import.meta.url), 'utf8'));

test('OAuth-Vertrag: mTLS, Formulare, Cache-Schutz und lokale Verweise sind konsistent', () => {
  validateServiceOAuth(source);
});

test('OAuth-Vertrag weist einen Endpunkt ohne Clientauthentisierung ab', () => {
  const document = structuredClone(source);
  document.paths['/oauth/token'].post.security = [];
  assert.throws(() => validateServiceOAuth(document), /Ungeschützter/);
});

test('OAuth-Vertrag weist Antworten mit erlaubtem Cache ab', () => {
  const document = structuredClone(source);
  document.paths['/oauth/introspect'].post.responses['200'].headers['Cache-Control'].schema.const = 'public';
  assert.throws(() => validateServiceOAuth(document), /Cache-Schutz/);
});

test('OAuth-Vertrag weist zusätzliche Informationen für inaktive Tokens ab', () => {
  const document = structuredClone(source);
  document.components.schemas.InactiveToken.properties.reason = { type: 'string' };
  assert.throws(() => validateServiceOAuth(document), /zusätzliche Informationen/);
});

test('OAuth-Vertrag weist defekte Verweise und produktive Servervorgaben ab', () => {
  const document = structuredClone(source);
  document.paths['/oauth/token'].post.responses['200'].content['application/json'].schema.$ref = '#/components/schemas/DoesNotExist';
  assert.throws(() => validateServiceOAuth(document), /Defekter/);
  const withServer = structuredClone(source);
  withServer.servers = [{ url: 'https://synthetic.test' }];
  assert.throws(() => validateServiceOAuth(withServer), /Servervorgabe/);
});

test('OAuth-Vertrag weist Problem Details an den OAuth-Protokollendpunkten ab', () => {
  const document = structuredClone(source);
  document.paths['/oauth/token'].post.responses['400'].content = { 'application/problem+json': {} };
  assert.throws(() => validateServiceOAuth(document), /Fehlerformat/);
});
