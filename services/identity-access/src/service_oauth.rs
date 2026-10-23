//! Wire models only. Parsed client IDs and scopes are claims, never authentication.
//! No HTTP handler, current registration check, issuance or protected action runs here.
use crate::service_tokens::{ActiveServiceToken, ServiceName, ServiceScope, ServiceToken};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use percent_encoding::percent_decode;
use serde_json::{Value, json};
use std::fmt;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OAuthError {
    InvalidRequest,
    InvalidClient,
    UnsupportedGrantType,
    InvalidScope,
}
impl OAuthError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::InvalidClient => "invalid_client",
            Self::UnsupportedGrantType => "unsupported_grant_type",
            Self::InvalidScope => "invalid_scope",
        }
    }
    pub fn response(self) -> Value {
        json!({ "error": self.code() })
    }
}
impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for OAuthError {}

/// Explicit parser limits, not productive rate, timeout or concurrency policy.
#[derive(Clone, Copy, Debug)]
pub struct FormLimits {
    bytes: usize,
    parameters: usize,
}
impl FormLimits {
    pub fn new(bytes: usize, parameters: usize) -> Result<Self, OAuthError> {
        if bytes == 0 || parameters == 0 {
            return Err(OAuthError::InvalidRequest);
        }
        Ok(Self { bytes, parameters })
    }
}

struct Form(Vec<(Zeroizing<String>, Zeroizing<String>)>);
impl Form {
    fn parse(body: &[u8], limits: FormLimits) -> Result<Self, OAuthError> {
        if body.len() > limits.bytes
            || body.split(|b| *b == b'&').filter(|p| !p.is_empty()).count() > limits.parameters
        {
            return Err(OAuthError::InvalidRequest);
        }
        let mut fields: Vec<(Zeroizing<String>, Zeroizing<String>)> = Vec::new();
        for pair in body.split(|b| *b == b'&').filter(|p| !p.is_empty()) {
            let mut parts = pair.splitn(2, |b| *b == b'=');
            let key = component(parts.next().ok_or(OAuthError::InvalidRequest)?)?;
            let value = component(parts.next().unwrap_or_default())?;
            if fields.iter().any(|(name, _)| name.as_str() == key.as_str()) {
                return Err(OAuthError::InvalidRequest);
            }
            fields.push((key, value));
        }
        let form = Self(fields);
        if ["client_secret", "client_assertion", "client_assertion_type"]
            .iter()
            .any(|key| form.get(key).is_some())
        {
            return Err(OAuthError::InvalidRequest);
        }
        Ok(form)
    }
    fn get(&self, name: &str) -> Option<&str> {
        // Small explicitly bounded form. Avoid a nonzeroizing duplicate of a decoded key.
        self.0
            .iter()
            .find(|(key, _)| key.as_str() == name)
            .map(|(_, value)| value.as_str())
            .filter(|v| !v.is_empty())
    }
    fn required(&self, name: &str) -> Result<&str, OAuthError> {
        self.get(name).ok_or(OAuthError::InvalidRequest)
    }
    fn client(&self) -> Result<ServiceName, OAuthError> {
        ServiceName::parse(self.required("client_id")?).map_err(|_| OAuthError::InvalidClient)
    }
}
fn component(raw: &[u8]) -> Result<Zeroizing<String>, OAuthError> {
    for (i, byte) in raw.iter().enumerate() {
        if *byte == b'%'
            && (!raw.get(i + 1).is_some_and(u8::is_ascii_hexdigit)
                || !raw.get(i + 2).is_some_and(u8::is_ascii_hexdigit))
        {
            return Err(OAuthError::InvalidRequest);
        }
    }
    let spaces = Zeroizing::new(
        raw.iter()
            .map(|b| if *b == b'+' { b' ' } else { *b })
            .collect::<Vec<_>>(),
    );
    let decoded = Zeroizing::new(percent_decode(&spaces).collect::<Vec<_>>());
    let text = std::str::from_utf8(&decoded).map_err(|_| OAuthError::InvalidRequest)?;
    Ok(Zeroizing::new(text.to_owned()))
}

/// A syntactically valid request. The caller must authenticate and authorize it separately.
#[derive(Debug)]
pub struct TokenRequest {
    client: ServiceName,
    scope: ServiceScope,
}
impl TokenRequest {
    pub fn parse(body: &[u8], limits: FormLimits) -> Result<Self, OAuthError> {
        let form = Form::parse(body, limits)?;
        let client = form.client()?;
        if form.required("grant_type")? != "client_credentials" {
            return Err(OAuthError::UnsupportedGrantType);
        }
        let scope = ServiceScope::parse(form.get("scope").ok_or(OAuthError::InvalidScope)?)
            .map_err(|_| OAuthError::InvalidScope)?;
        Ok(Self { client, scope })
    }
    pub fn claimed_client(&self) -> ServiceName {
        self.client
    }
    pub fn scope(&self) -> ServiceScope {
        self.scope
    }
    pub fn target(&self) -> ServiceName {
        match self.scope {
            ServiceScope::MembershipRead => ServiceName::OrganizationConfig,
            ServiceScope::LocalLogin | ServiceScope::SessionCheck => ServiceName::IdentityAccess,
        }
    }
}

#[derive(Debug)]
pub struct IntrospectionRequest {
    client: ServiceName,
    token: Option<ServiceToken>,
}
impl IntrospectionRequest {
    pub fn parse(body: &[u8], limits: FormLimits) -> Result<Self, OAuthError> {
        let form = Form::parse(body, limits)?;
        Ok(Self {
            client: form.client()?,
            token: decode_token(form.required("token")?),
        })
    }
    pub fn claimed_client(&self) -> ServiceName {
        self.client
    }
    /// None means an unrepresentable token, NOT an unauthenticated introspection result.
    /// Authenticate the actual resource peer before responding, including for this case.
    pub fn token(&self) -> Option<&ServiceToken> {
        self.token.as_ref()
    }
}

pub fn encode_token(token: &ServiceToken) -> Zeroizing<String> {
    Zeroizing::new(URL_SAFE_NO_PAD.encode(token.expose()))
}
pub fn decode_token(text: &str) -> Option<ServiceToken> {
    if text.len() != 43 {
        return None;
    }
    let mut raw = Zeroizing::new([0u8; 32]);
    let count = URL_SAFE_NO_PAD.decode_slice(text, raw.as_mut()).ok()?;
    if count != 32 {
        return None;
    }
    ServiceToken::from_bytes(raw.as_ref())
}

/// Serializes already checked internal metadata; it does NOT authenticate or refresh it.
/// Recheck transport/registration and time after waits. Never positively cache this value.
pub fn introspection_response(active: Option<&ActiveServiceToken>) -> Value {
    match active {
        None => json!({ "active": false }),
        Some(token) => json!({
            "active": true,
            "client_id": token.client().text(),
            "scope": token.scope().text(),
            "iss": token.issuer(),
            "aud": token.target().text(),
            "iat": token.created_at(),
            "exp": token.expires_at(),
            "cnf": { "x5t#S256": URL_SAFE_NO_PAD.encode(token.certificate()) }
        }),
    }
}
