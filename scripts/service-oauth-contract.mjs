// Static project invariants, not a replacement for a complete OpenAPI validator.
export function validateServiceOAuth(document) {
  if (document.openapi !== '3.1.1' || !document.info?.version) throw new Error('OpenAPI-Version fehlt.');
  if (document.servers?.length) throw new Error('Keine produktive Servervorgabe im Dienstvertrag.');
  if (document.components?.securitySchemes?.serviceMTLS?.type !== 'mutualTLS') throw new Error('mTLS fehlt.');
  const protectedByMTLS = security => Array.isArray(security) && security.length === 1 &&
    Object.keys(security[0]).length === 1 && Array.isArray(security[0].serviceMTLS) && security[0].serviceMTLS.length === 0;
  for (const path of ['/oauth/token', '/oauth/introspect']) {
    const operation = document.paths?.[path]?.post;
    if (!operation || !protectedByMTLS(operation.security ?? document.security)) throw new Error('Ungeschützter OAuth-Endpunkt.');
    const form = operation.requestBody?.content?.['application/x-www-form-urlencoded']?.schema;
    if (!operation.requestBody?.required || !form?.required?.includes('client_id')) throw new Error('Clientkennung im Formular fehlt.');
    for (const response of Object.values(operation.responses)) {
      if (response.headers?.['Cache-Control']?.schema?.const !== 'no-store' ||
          response.headers?.Pragma?.schema?.const !== 'no-cache') throw new Error('Cache-Schutz fehlt.');
      if (response.content && Object.keys(response.content).some(type => type !== 'application/json')) throw new Error('Falsches OAuth-Fehlerformat.');
    }
  }
  const inactive = document.components.schemas.InactiveToken;
  if (inactive.additionalProperties !== false || inactive.properties?.active?.const !== false ||
      JSON.stringify(inactive.required) !== '["active"]' || Object.keys(inactive.properties).length !== 1) {
    throw new Error('Inaktive Antwort enthält zusätzliche Informationen.');
  }
  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if ('$ref' in node) {
      if (typeof node.$ref !== 'string' || !node.$ref.startsWith('#/')) throw new Error('Nur lokale Vertragsverweise zulässig.');
      const target = node.$ref.slice(2).split('/').reduce((value, key) => value?.[key.replaceAll('~1', '/').replaceAll('~0', '~')], document);
      if (!target) throw new Error('Defekter OpenAPI-Verweis.');
    }
    for (const value of Object.values(node)) walk(value);
  }
  walk(document);
}
