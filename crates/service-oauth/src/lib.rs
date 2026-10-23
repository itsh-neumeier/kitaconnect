//! Pure response validation, never transport authentication or permission to act.
//! Expectations must come from trusted configuration and the actual caller connection.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::fmt;
use zeroize::Zeroizing;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResponseError;
impl fmt::Debug for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuthResponseRejected")
    }
}
impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OAuth response rejected")
    }
}
impl std::error::Error for ResponseError {}

#[derive(Clone, Copy, Debug)]
pub struct ResponseLimits {
    bytes: usize,
}
impl ResponseLimits {
    pub fn new(bytes: usize) -> Result<Self, ResponseError> {
        if bytes == 0 {
            return Err(ResponseError);
        }
        Ok(Self { bytes })
    }
}

// Bound nesting even in ignored extensions. Strings/escapes are validated by Serde later.
fn object<T: DeserializeOwned>(
    status: u16,
    body: &[u8],
    limits: ResponseLimits,
) -> Result<T, ResponseError> {
    if status != 200
        || body.len() > limits.bytes
        || body.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{')
    {
        return Err(ResponseError);
    }
    let (mut quoted, mut escaped, mut depth) = (false, false, 0usize);
    for byte in body {
        if quoted {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > 64 {
                        return Err(ResponseError);
                    }
                }
                b'}' | b']' => depth = depth.checked_sub(1).ok_or(ResponseError)?,
                _ => {}
            }
        }
    }
    serde_json::from_slice(body).map_err(|_| ResponseError)
}

fn forbidden<'de, D: Deserializer<'de>>(_: D) -> Result<Option<()>, D::Error> {
    Err(serde::de::Error::custom("unsupported response field"))
}
// Reject explicit null too. Otherwise Option's null-as-missing permits duplicate known fields.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
struct Secret(Zeroizing<String>);
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(|s| Self(Zeroizing::new(s)))
    }
}
fn canonical(text: &str) -> Result<Zeroizing<[u8; 32]>, ResponseError> {
    if text.len() != 43 {
        return Err(ResponseError);
    }
    let mut raw = Zeroizing::new([0; 32]);
    if URL_SAFE_NO_PAD
        .decode_slice(text, raw.as_mut())
        .map_err(|_| ResponseError)?
        != 32
    {
        return Err(ResponseError);
    }
    Ok(raw)
}

#[derive(Deserialize)]
struct RawToken {
    access_token: Secret,
    token_type: String,
    expires_in: u64,
    scope: String,
    #[serde(default, rename = "error", deserialize_with = "forbidden")]
    _error: Option<()>,
    #[serde(default, rename = "refresh_token", deserialize_with = "forbidden")]
    _refresh: Option<()>,
}

/// Syntactically checked issuance response, not evidence of current token activity.
pub struct TokenResponse {
    token: Zeroizing<String>,
    expires_in: u64,
}
impl fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenResponse(REDACTED)")
    }
}
impl TokenResponse {
    pub fn parse(
        status: u16,
        body: &[u8],
        expected_scope: &str,
        limits: ResponseLimits,
    ) -> Result<Self, ResponseError> {
        let raw: RawToken = object(status, body, limits)?;
        if expected_scope.is_empty()
            || raw.scope != expected_scope
            || !raw.token_type.eq_ignore_ascii_case("Bearer")
            || !(1..=300).contains(&raw.expires_in)
        {
            return Err(ResponseError);
        }
        canonical(&raw.access_token.0)?;
        Ok(Self {
            token: raw.access_token.0,
            expires_in: raw.expires_in,
        })
    }
    /// Secret: only place in the bounded outgoing form; never log it.
    pub fn expose_token(&self) -> &str {
        &self.token
    }
    /// Received duration only, not a local expiration or permission cache.
    pub fn expires_in(&self) -> u64 {
        self.expires_in
    }
}

#[derive(Deserialize)]
struct Confirmation {
    #[serde(rename = "x5t#S256")]
    certificate: String,
}
// Serde struct derives can also accept a positional JSON array. Only an object is allowed here.
struct Object<T>(T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct MapOnly<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for MapOnly<T> {
            type Value = Object<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(Object)
            }
        }
        d.deserialize_map(MapOnly(std::marker::PhantomData))
    }
}
#[derive(Deserialize)]
struct RawIntrospection {
    active: bool,
    #[serde(default, deserialize_with = "present")]
    client_id: Option<String>,
    #[serde(default, deserialize_with = "present")]
    iss: Option<String>,
    #[serde(default, deserialize_with = "present")]
    aud: Option<String>,
    #[serde(default, deserialize_with = "present")]
    scope: Option<String>,
    #[serde(default, deserialize_with = "present")]
    iat: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    exp: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    cnf: Option<Object<Confirmation>>,
    #[serde(default, rename = "error", deserialize_with = "forbidden")]
    _error: Option<()>,
}

/// Supply after the RPC from trusted config, actual incoming TLS and a fresh trusted clock.
/// Never populate from browser claims. This type does not itself prove their provenance.
pub struct ExpectedActive<'a> {
    pub issuer: &'a str,
    pub client: &'a str,
    pub target: &'a str,
    pub scope: &'a str,
    pub certificate: [u8; 32],
    pub now: u64,
}

/// Validated snapshot only. Re-run introspection for the next protected request.
#[derive(Debug)]
pub enum IntrospectionResponse {
    Inactive,
    Active(ActiveSnapshot),
}
#[derive(Debug)]
pub struct ActiveSnapshot {
    created: u64,
    expires: u64,
}
impl ActiveSnapshot {
    pub fn created_at(&self) -> u64 {
        self.created
    }
    pub fn expires_at(&self) -> u64 {
        self.expires
    }
}
impl IntrospectionResponse {
    pub fn parse(
        status: u16,
        body: &[u8],
        expected: ExpectedActive<'_>,
        limits: ResponseLimits,
    ) -> Result<Self, ResponseError> {
        let raw: RawIntrospection = object(status, body, limits)?;
        if !raw.active {
            return Ok(Self::Inactive);
        }
        for (value, wanted) in [
            (raw.iss.as_deref(), expected.issuer),
            (raw.client_id.as_deref(), expected.client),
            (raw.aud.as_deref(), expected.target),
            (raw.scope.as_deref(), expected.scope),
        ] {
            if wanted.is_empty() || value != Some(wanted) {
                return Err(ResponseError);
            }
        }
        let created = raw.iat.ok_or(ResponseError)?;
        let expires = raw.exp.ok_or(ResponseError)?;
        let lifetime = expires.checked_sub(created).ok_or(ResponseError)?;
        if !(1..=300).contains(&lifetime) || expected.now < created || expected.now >= expires {
            return Err(ResponseError);
        }
        let cnf = raw.cnf.ok_or(ResponseError)?;
        if *canonical(&cnf.0.certificate)? != expected.certificate {
            return Err(ResponseError);
        }
        Ok(Self::Active(ActiveSnapshot { created, expires }))
    }
}
