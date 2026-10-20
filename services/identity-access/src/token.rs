//! Random bearer material in memory only; never an internal record key.
use sha2::{Digest, Sha256};
use std::fmt;
use zeroize::Zeroizing;

pub struct SessionToken(Zeroizing<[u8; 32]>);

impl SessionToken {
    pub fn generate() -> Result<Self, getrandom::Error> {
        let mut bytes = Zeroizing::new([0; 32]);
        getrandom::fill(&mut *bytes)?;
        Ok(Self(bytes))
    }

    /// Explicit transport boundary; the caller must not log or persist these bytes.
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }

    /// Only the fixed-size binary form is accepted; HTTP encoding is a later contract.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }
        let mut value = Zeroizing::new([0; 32]);
        value.copy_from_slice(bytes);
        Some(Self(value))
    }

    pub(crate) fn fingerprint(&self) -> [u8; 32] {
        Sha256::digest(self.expose()).into()
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionToken([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_transport_is_strict_and_debug_is_redacted() {
        let token = SessionToken::generate().unwrap();
        assert_eq!(format!("{token:?}"), "SessionToken([REDACTED])");
        assert!(SessionToken::from_bytes(&token.expose()[..31]).is_none());
        let restored = SessionToken::from_bytes(token.expose()).unwrap();
        assert_eq!(token.fingerprint(), restored.fingerprint());
    }
}
