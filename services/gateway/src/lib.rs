//! Browser syntax boundary only. Presented secrets never establish authentication or CSRF authority.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hyper::{
    Method, Uri,
    header::{self, HeaderMap, HeaderValue},
};
use std::{collections::HashSet, fmt};
use zeroize::Zeroizing;

#[derive(Clone, Copy)]
pub struct BoundaryError;
impl fmt::Debug for BoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("browser request rejected")
    }
}
impl fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for BoundaryError {}

pub struct PresentedSecret(Zeroizing<[u8; 32]>);
impl fmt::Debug for PresentedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PresentedSecret(REDACTED)")
    }
}
impl PresentedSecret {
    pub fn from_bytes(value: &[u8]) -> Result<Self, BoundaryError> {
        if value.len() != 32 {
            return Err(BoundaryError);
        }
        let mut bytes = Zeroizing::new([0; 32]);
        bytes.copy_from_slice(value);
        Ok(Self(bytes))
    }
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }
    fn parse(value: &str) -> Result<Self, BoundaryError> {
        if value.len() != 43 {
            return Err(BoundaryError);
        }
        let mut bytes = Zeroizing::new([0; 32]);
        if URL_SAFE_NO_PAD
            .decode_slice(value, bytes.as_mut())
            .map_err(|_| BoundaryError)?
            != 32
        {
            return Err(BoundaryError);
        }
        let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(bytes.as_slice()));
        if encoded.as_str() != value {
            return Err(BoundaryError);
        }
        Ok(Self(bytes))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CookieKind {
    Context,
    Session,
}
impl CookieKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Context => "__Host-kc_context",
            Self::Session => "__Host-kc_session",
        }
    }
    pub fn issue(self, secret: &PresentedSecret) -> Result<HeaderValue, BoundaryError> {
        let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(secret.expose()));
        let raw = Zeroizing::new(format!(
            "{}={}; Path=/; Secure; HttpOnly; SameSite=Strict",
            self.name(),
            encoded.as_str()
        ));
        let mut value = HeaderValue::from_str(&raw).map_err(|_| BoundaryError)?;
        value.set_sensitive(true);
        Ok(value)
    }
    pub fn clear(self) -> HeaderValue {
        let mut value = HeaderValue::from_str(&format!(
            "{}=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0",
            self.name()
        ))
        .expect("fixed cookie deletion profile");
        value.set_sensitive(true);
        value
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Context,
    LoginStart,
    SessionCheck,
    Logout,
    Rotate,
}
impl Operation {
    pub fn path(self) -> &'static str {
        match self {
            Self::Context => "/api/v1/auth/context",
            Self::LoginStart => "/api/v1/auth/login/start",
            Self::SessionCheck => "/api/v1/auth/session/check",
            Self::Logout => "/api/v1/auth/session/logout",
            Self::Rotate => "/api/v1/auth/session/rotate",
        }
    }
    fn cookie(self) -> Option<CookieKind> {
        match self {
            Self::Context => None,
            Self::LoginStart => Some(CookieKind::Context),
            _ => Some(CookieKind::Session),
        }
    }
}
pub struct PresentedRequest {
    operation: Operation,
    credential: Option<PresentedSecret>,
    csrf: Option<PresentedSecret>,
}
impl fmt::Debug for PresentedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PresentedRequest(REDACTED)")
    }
}
impl PresentedRequest {
    pub fn operation(&self) -> Operation {
        self.operation
    }
    pub fn credential(&self) -> Option<&PresentedSecret> {
        self.credential.as_ref()
    }
    pub fn csrf(&self) -> Option<&PresentedSecret> {
        self.csrf.as_ref()
    }
}
#[derive(Clone, Copy)]
pub struct HeaderLimits {
    pub count: usize,
    pub bytes: usize,
    pub cookie_bytes: usize,
}
pub struct BrowserBoundary {
    origin: String,
    authority: String,
    limits: HeaderLimits,
}
fn single<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, BoundaryError> {
    let mut values = headers.get_all(name).iter();
    let value = values.next();
    if values.next().is_some() {
        return Err(BoundaryError);
    }
    value
        .map(|v| v.to_str().map_err(|_| BoundaryError))
        .transpose()
}
fn cookie_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
fn cookie_value(value: &str) -> bool {
    value.bytes().all(|b| {
        b == 0x21
            || (0x23..=0x2b).contains(&b)
            || (0x2d..=0x3a).contains(&b)
            || (0x3c..=0x5b).contains(&b)
            || (0x5d..=0x7e).contains(&b)
    })
}
fn cookies(
    headers: &HeaderMap,
    kind: Option<CookieKind>,
    max: usize,
) -> Result<Option<PresentedSecret>, BoundaryError> {
    let Some(raw) = single(headers, "cookie")? else {
        return if kind.is_none() {
            Ok(None)
        } else {
            Err(BoundaryError)
        };
    };
    if raw.is_empty() || raw.len() > max {
        return Err(BoundaryError);
    }
    let mut names = HashSet::new();
    let mut found = None;
    for pair in raw.split(';') {
        let pair = pair.trim_start_matches(' ');
        let (name, value) = pair.split_once('=').ok_or(BoundaryError)?;
        if !cookie_name(name) || !cookie_value(value) || !names.insert(name) {
            return Err(BoundaryError);
        }
        if kind.is_some_and(|k| k.name() == name) {
            found = Some(PresentedSecret::parse(value)?);
        }
    }
    if kind.is_some() && found.is_none() {
        return Err(BoundaryError);
    }
    Ok(found)
}
impl BrowserBoundary {
    pub fn new(origin: &str, limits: HeaderLimits) -> Result<Self, BoundaryError> {
        if limits.count == 0 || limits.bytes == 0 || limits.cookie_bytes == 0 {
            return Err(BoundaryError);
        }
        let parsed = url::Url::parse(origin).map_err(|_| BoundaryError)?;
        if parsed.scheme() != "https"
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || parsed.origin().ascii_serialization() != origin
        {
            return Err(BoundaryError);
        }
        let authority = origin
            .strip_prefix("https://")
            .ok_or(BoundaryError)?
            .to_owned();
        Ok(Self {
            origin: origin.to_owned(),
            authority,
            limits,
        })
    }
    /// Syntax and configured origin only. Identity must verify current server-side binding afterwards.
    pub fn present(
        &self,
        operation: Operation,
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
    ) -> Result<PresentedRequest, BoundaryError> {
        if method != Method::POST
            || uri.scheme().is_some()
            || uri.authority().is_some()
            || uri.query().is_some()
            || uri.path() != operation.path()
            || headers.len() > self.limits.count
            || headers.contains_key(header::AUTHORIZATION)
        {
            return Err(BoundaryError);
        }
        let mut bytes = 2usize;
        for (name, value) in headers {
            bytes = bytes
                .checked_add(name.as_str().len())
                .and_then(|n| n.checked_add(value.as_bytes().len()))
                .and_then(|n| n.checked_add(4))
                .ok_or(BoundaryError)?;
            if bytes > self.limits.bytes {
                return Err(BoundaryError);
            }
        }
        if single(headers, "origin")? != Some(self.origin.as_str())
            || single(headers, "host")? != Some(self.authority.as_str())
        {
            return Err(BoundaryError);
        }
        let credential = cookies(headers, operation.cookie(), self.limits.cookie_bytes)?;
        let csrf = match (operation.cookie(), single(headers, "x-kc-csrf")?) {
            (Some(_), Some(value)) => Some(PresentedSecret::parse(value)?),
            (None, None) => None,
            _ => return Err(BoundaryError),
        };
        Ok(PresentedRequest {
            operation,
            credential,
            csrf,
        })
    }
}
