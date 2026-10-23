use kitaconnect_identity_access::{
    service_oauth::{
        FormLimits, IntrospectionRequest, OAuthError, TokenRequest, decode_token, encode_token,
        introspection_response,
    },
    service_tokens::{ServiceName, ServiceScope, ServiceToken},
};

fn limits() -> FormLimits {
    // Isolated parser values only, not a productive policy.
    FormLimits::new(1024, 8).unwrap()
}
const TOKEN_FORM: &[u8] = b"client_id=gateway&grant_type=client_credentials&scope=session%3Acheck";

#[test]
fn form_parsing_preserves_claims_without_granting_permissions() {
    let request = TokenRequest::parse(TOKEN_FORM, limits()).unwrap();
    assert_eq!(request.claimed_client(), ServiceName::Gateway);
    assert_eq!(request.scope(), ServiceScope::SessionCheck);
    assert_eq!(request.target(), ServiceName::IdentityAccess);
    let request = TokenRequest::parse(
        b"client_id=organization-config&grant_type=client_credentials&scope=membership:read",
        limits(),
    )
    .unwrap();
    // This pairing has NO grant in the current registry. Parsing is not authorization.
    assert_eq!(request.claimed_client(), ServiceName::OrganizationConfig);
    assert_eq!(request.target(), ServiceName::OrganizationConfig);
    let request = TokenRequest::parse(
        b"client_id=gateway&grant_type=client_credentials&scope=login:local&extension=%E2%82%AC+test",
        limits(),
    )
    .unwrap();
    assert_eq!(request.scope(), ServiceScope::LocalLogin);
}

#[test]
fn duplicate_names_bad_encodings_and_alternative_credentials_are_rejected() {
    for suffix in [
        "&client_id=gateway",
        "&%63lient_id=identity-access",
        "&scope=session:check",
        "&extension=a&%65xtension=b",
        "&extension=%",
        "&extension=%0",
        "&extension=%gg",
        "&extension=%FF",
        "&extension=%C0%AF",
        "&client_secret=not-an-accepted-authentication-method",
        "&client_assertion=not-an-accepted-authentication-method",
        "&client_assertion_type=not-an-accepted-authentication-method",
    ] {
        let mut body = TOKEN_FORM.to_vec();
        body.extend_from_slice(suffix.as_bytes());
        assert_eq!(
            TokenRequest::parse(&body, limits()).unwrap_err(),
            OAuthError::InvalidRequest
        );
    }
    let mut body = TOKEN_FORM.to_vec();
    body.extend_from_slice(b"&extension=\xff");
    assert_eq!(
        TokenRequest::parse(&body, limits()).unwrap_err(),
        OAuthError::InvalidRequest
    );
}

#[test]
fn missing_empty_unknown_and_multiple_scopes_have_protocol_errors() {
    for (body, expected) in [
        ("", OAuthError::InvalidRequest),
        (
            "client_id=&grant_type=client_credentials&scope=session:check",
            OAuthError::InvalidRequest,
        ),
        (
            "client_id=gateway&grant_type=&scope=session:check",
            OAuthError::InvalidRequest,
        ),
        (
            "client_id=unknown&grant_type=client_credentials&scope=session:check",
            OAuthError::InvalidClient,
        ),
        (
            "client_id=gateway&grant_type=password&scope=session:check",
            OAuthError::UnsupportedGrantType,
        ),
        (
            "client_id=gateway&grant_type=client_credentials",
            OAuthError::InvalidScope,
        ),
        (
            "client_id=gateway&grant_type=client_credentials&scope=",
            OAuthError::InvalidScope,
        ),
        (
            "client_id=gateway&grant_type=client_credentials&scope=unknown",
            OAuthError::InvalidScope,
        ),
        (
            "client_id=gateway&grant_type=client_credentials&scope=session:check+login:local",
            OAuthError::InvalidScope,
        ),
    ] {
        assert_eq!(
            TokenRequest::parse(body.as_bytes(), limits()).unwrap_err(),
            expected
        );
    }
    let mut body = TOKEN_FORM.to_vec();
    body.extend_from_slice(b"&client_secret=&unknown");
    assert!(TokenRequest::parse(&body, limits()).is_ok());
}

#[test]
fn configured_limits_are_exact_and_required() {
    assert!(FormLimits::new(0, 1).is_err());
    assert!(FormLimits::new(1, 0).is_err());
    assert!(TokenRequest::parse(TOKEN_FORM, FormLimits::new(TOKEN_FORM.len(), 3).unwrap()).is_ok());
    assert_eq!(
        TokenRequest::parse(
            TOKEN_FORM,
            FormLimits::new(TOKEN_FORM.len() - 1, 3).unwrap()
        )
        .unwrap_err(),
        OAuthError::InvalidRequest
    );
    assert_eq!(
        TokenRequest::parse(TOKEN_FORM, FormLimits::new(TOKEN_FORM.len(), 2).unwrap()).unwrap_err(),
        OAuthError::InvalidRequest
    );
    let mut body = TOKEN_FORM.to_vec();
    body.extend_from_slice(b"&extension=1");
    assert!(TokenRequest::parse(&body, FormLimits::new(body.len(), 3).unwrap()).is_err());
}

#[test]
fn tokens_have_one_canonical_url_safe_representation_and_redacted_debug() {
    for byte in 0..=u8::MAX {
        let token = ServiceToken::from_bytes(&[byte; 32]).unwrap();
        let encoded = encode_token(&token);
        assert_eq!(encoded.len(), 43);
        assert!(
            encoded
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
        let decoded = decode_token(&encoded).unwrap();
        assert!(decoded.expose() == token.expose());
        assert!(!format!("{decoded:?}").contains(encoded.as_str()));
        assert!(decode_token(&format!("{}=", encoded.as_str())).is_none());
        assert!(decode_token(&encoded[..42]).is_none());
        // The final two bits are unused and must be zero, not silently discarded.
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let last = alphabet
            .iter()
            .position(|c| *c == encoded.as_bytes()[42])
            .unwrap();
        let mut alternate = encoded.as_bytes().to_vec();
        alternate[42] = alphabet[last + 1];
        assert!(decode_token(std::str::from_utf8(&alternate).unwrap()).is_none());
    }
}

#[test]
fn introspection_keeps_unrepresentable_tokens_separate_from_invalid_requests() {
    for candidate in ["unknown", "", "not/a/token", "not+a+token"] {
        let body = format!("client_id=organization-config&token={candidate}");
        if candidate.is_empty() {
            assert_eq!(
                IntrospectionRequest::parse(body.as_bytes(), limits()).unwrap_err(),
                OAuthError::InvalidRequest
            );
        } else {
            let request = IntrospectionRequest::parse(body.as_bytes(), limits()).unwrap();
            assert_eq!(request.claimed_client(), ServiceName::OrganizationConfig);
            assert!(request.token().is_none());
        }
    }
    let token = ServiceToken::from_bytes(&[8; 32]).unwrap();
    let encoded = encode_token(&token);
    let body = format!(
        "client_id=organization-config&token={}&token_type_hint=unexpected",
        encoded.as_str()
    );
    let request = IntrospectionRequest::parse(body.as_bytes(), limits()).unwrap();
    assert!(request.token().unwrap().expose() == token.expose());
    assert!(!format!("{request:?}").contains(encoded.as_str()));
    assert!(IntrospectionRequest::parse(b"client_id=organization-config", limits()).is_err());
    assert!(
        IntrospectionRequest::parse(b"client_id=organization-config&token=a&%74oken=b", limits())
            .is_err()
    );
}

#[test]
fn inactive_and_error_responses_do_not_leak_diagnostics_or_claims() {
    let inactive = introspection_response(None);
    assert_eq!(inactive.as_object().unwrap().len(), 1);
    assert_eq!(inactive["active"], false);
    for error in [
        OAuthError::InvalidClient,
        OAuthError::InvalidRequest,
        OAuthError::InvalidScope,
        OAuthError::UnsupportedGrantType,
    ] {
        let value = error.response();
        assert_eq!(value.as_object().unwrap().len(), 1);
        assert_eq!(value["error"], error.code());
        assert_eq!(error.to_string(), error.code());
    }
}
