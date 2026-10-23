use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kitaconnect_service_oauth::{
    ExpectedActive, IntrospectionResponse, ResponseLimits, TokenResponse,
};
use serde_json::{Value, json};

fn limits() -> ResponseLimits {
    ResponseLimits::new(4096).unwrap()
}
fn token() -> Value {
    json!({"access_token": URL_SAFE_NO_PAD.encode([7;32]), "token_type":"Bearer", "expires_in":120,"scope":"session:check"})
}
fn active() -> Value {
    json!({"active":true,"client_id":"gateway","iss":"https://identity.test/","aud":"identity-access","scope":"session:check","iat":100,"exp":220,"cnf":{"x5t#S256":URL_SAFE_NO_PAD.encode([8;32])}})
}
fn expected(now: u64) -> ExpectedActive<'static> {
    ExpectedActive {
        issuer: "https://identity.test/",
        client: "gateway",
        target: "identity-access",
        scope: "session:check",
        certificate: [8; 32],
        now,
    }
}
fn parse_token(v: &Value) -> Result<TokenResponse, kitaconnect_service_oauth::ResponseError> {
    TokenResponse::parse(
        200,
        &serde_json::to_vec(v).unwrap(),
        "session:check",
        limits(),
    )
}
fn parse_active(
    v: &Value,
) -> Result<IntrospectionResponse, kitaconnect_service_oauth::ResponseError> {
    IntrospectionResponse::parse(
        200,
        &serde_json::to_vec(v).unwrap(),
        expected(150),
        limits(),
    )
}
fn duplicate(v: &Value, key: &str, encoded_key: &str) -> String {
    let mut text = serde_json::to_string(v).unwrap();
    text.pop();
    format!("{text},\"{encoded_key}\":{}}}", v[key])
}

#[test]
fn issuance_is_canonical_bounded_and_redacted() {
    for lifetime in [1, 300] {
        let mut value = token();
        value["expires_in"] = json!(lifetime);
        value["token_type"] = json!("bEaReR");
        value["extension"] = json!({"nested":[null,true,7,"quotes \\\" [] {}"]});
        let parsed = parse_token(&value).unwrap();
        assert_eq!(parsed.expose_token(), URL_SAFE_NO_PAD.encode([7; 32]));
        assert_eq!(parsed.expires_in(), lifetime);
        assert_eq!(format!("{parsed:?}"), "TokenResponse(REDACTED)");
    }
    for field in ["access_token", "token_type", "expires_in", "scope"] {
        let mut value = token();
        value.as_object_mut().unwrap().remove(field);
        assert!(parse_token(&value).is_err());
        value[field] = Value::Null;
        assert!(parse_token(&value).is_err());
    }
    for (key, bad) in [
        ("token_type", json!("Basic")),
        ("scope", json!("session:check login:local")),
        ("expires_in", json!(0)),
        ("expires_in", json!(301)),
        ("expires_in", json!(-1)),
        ("expires_in", json!(1.0)),
        ("expires_in", json!("120")),
        ("expires_in", json!(u64::MAX)),
        ("access_token", json!("A".repeat(42))),
        ("access_token", json!(format!("{}=", "A".repeat(43)))),
        ("access_token", json!(format!("{}B", "A".repeat(42)))),
        ("access_token", json!(format!("{}+", "A".repeat(42)))),
        ("error", Value::Null),
        ("error", json!("invalid_client")),
        ("refresh_token", Value::Null),
    ] {
        let mut value = token();
        value[key] = bad;
        assert!(parse_token(&value).is_err(), "{key}");
    }
}

#[test]
fn known_duplicates_are_not_silently_overwritten() {
    for (value, fields) in [
        (
            token(),
            vec!["access_token", "token_type", "expires_in", "scope"],
        ),
        (
            active(),
            vec![
                "active",
                "client_id",
                "iss",
                "aud",
                "scope",
                "iat",
                "exp",
                "cnf",
            ],
        ),
    ] {
        for field in fields {
            for encoded in [
                field.to_owned(),
                format!("\\u{:04x}{}", field.as_bytes()[0], &field[1..]),
            ] {
                let body = duplicate(&value, field, &encoded);
                if value.get("active").is_some() {
                    assert!(
                        IntrospectionResponse::parse(200, body.as_bytes(), expected(150), limits())
                            .is_err()
                    );
                } else {
                    assert!(
                        TokenResponse::parse(200, body.as_bytes(), "session:check", limits())
                            .is_err()
                    );
                }
            }
        }
    }
    let mut v = active();
    v["cnf"] = json!({"x5t#S256":"ignored"});
    let text = serde_json::to_string(&v).unwrap().replace(
        "\"x5t#S256\":\"ignored\"",
        &format!(
            "\"x5t#S256\":\"ignored\",\"x5t#S256\":\"{}\"",
            URL_SAFE_NO_PAD.encode([8; 32])
        ),
    );
    assert!(IntrospectionResponse::parse(200, text.as_bytes(), expected(150), limits()).is_err());
    for field in ["iss", "client_id", "aud", "scope", "iat", "exp", "cnf"] {
        let text = serde_json::to_string(&active()).unwrap();
        let text = format!("{{\"{field}\":null,{}", &text[1..]);
        assert!(
            IntrospectionResponse::parse(200, text.as_bytes(), expected(150), limits()).is_err()
        );
    }
}

#[test]
fn active_requires_every_binding_and_exact_types() {
    let parsed = parse_active(&active()).unwrap();
    let IntrospectionResponse::Active(snapshot) = parsed else {
        panic!("expected active")
    };
    assert_eq!((snapshot.created_at(), snapshot.expires_at()), (100, 220));
    for field in [
        "active",
        "client_id",
        "iss",
        "aud",
        "scope",
        "iat",
        "exp",
        "cnf",
    ] {
        let mut value = active();
        value.as_object_mut().unwrap().remove(field);
        assert!(parse_active(&value).is_err(), "{field}");
        value[field] = Value::Null;
        assert!(parse_active(&value).is_err());
    }
    for (key, bad) in [
        ("active", json!("true")),
        ("client_id", json!("organization-config")),
        ("iss", json!("https://other.test/")),
        ("aud", json!(["identity-access"])),
        ("aud", json!("organization-config")),
        ("scope", json!("login:local")),
        ("iat", json!(-1)),
        ("iat", json!(100.0)),
        ("exp", json!("220")),
        ("cnf", json!([URL_SAFE_NO_PAD.encode([8; 32])])),
        ("cnf", json!({})),
        ("cnf", json!({"x5t#S256":URL_SAFE_NO_PAD.encode([9;32])})),
        ("cnf", json!({"x5t#S256":format!("{}h","I".repeat(42))})),
        ("error", Value::Null),
    ] {
        let mut value = active();
        value[key] = bad;
        assert!(parse_active(&value).is_err(), "{key}");
    }
    let mut e = expected(150);
    e.certificate = [9; 32];
    assert!(
        IntrospectionResponse::parse(200, &serde_json::to_vec(&active()).unwrap(), e, limits())
            .is_err()
    );
    for field in ["issuer", "client", "target", "scope"] {
        let mut e = expected(150);
        match field {
            "issuer" => e.issuer = "",
            "client" => e.client = "",
            "target" => e.target = "",
            _ => e.scope = "",
        }
        assert!(
            IntrospectionResponse::parse(200, &serde_json::to_vec(&active()).unwrap(), e, limits())
                .is_err()
        );
    }
}

#[test]
fn time_checks_include_boundaries_and_overflow() {
    let body = serde_json::to_vec(&active()).unwrap();
    for now in [100, 219] {
        assert!(IntrospectionResponse::parse(200, &body, expected(now), limits()).is_ok());
    }
    for now in [0, 99, 220, u64::MAX] {
        assert!(IntrospectionResponse::parse(200, &body, expected(now), limits()).is_err());
    }
    for (iat, exp) in [
        (100, 100),
        (101, 100),
        (100, 401),
        (0, u64::MAX),
        (u64::MAX, 0),
    ] {
        let mut value = active();
        value["iat"] = json!(iat);
        value["exp"] = json!(exp);
        assert!(parse_active(&value).is_err());
    }
    for (iat, exp, now) in [
        (100, 101, 100),
        (100, 400, 399),
        (u64::MAX - 1, u64::MAX, u64::MAX - 1),
    ] {
        let mut value = active();
        value["iat"] = json!(iat);
        value["exp"] = json!(exp);
        assert!(
            IntrospectionResponse::parse(
                200,
                &serde_json::to_vec(&value).unwrap(),
                expected(now),
                limits()
            )
            .is_ok()
        );
    }
}

#[test]
fn inactive_and_extensions_never_create_active_metadata() {
    for v in [
        json!({"active":false}),
        json!({"active":false,"iss":"other","extension":[1,2,3]}),
    ] {
        assert!(matches!(
            parse_active(&v).unwrap(),
            IntrospectionResponse::Inactive
        ));
    }
    for v in [
        json!({"active":false,"iat":"bad"}),
        json!({"active":false,"iss":null}),
        json!({"active":false,"error":"bad"}),
    ] {
        assert!(parse_active(&v).is_err());
    }
    let mut v = active();
    v["extension"] = json!({"arbitrary":[null,true,"ignored"]});
    v["cnf"]["extension"] = json!(true);
    assert!(parse_active(&v).is_ok());
}

#[test]
fn status_size_shape_and_depth_fail_closed_without_input_in_errors() {
    assert!(ResponseLimits::new(0).is_err());
    let body = serde_json::to_vec(&token()).unwrap();
    for status in [0, 201, 204, 302, 400, 401, 503] {
        assert!(TokenResponse::parse(status, &body, "session:check", limits()).is_err());
    }
    assert!(TokenResponse::parse(200, &body, "", limits()).is_err());
    assert!(
        TokenResponse::parse(
            200,
            &body,
            "session:check",
            ResponseLimits::new(body.len()).unwrap()
        )
        .is_ok()
    );
    assert!(
        TokenResponse::parse(
            200,
            &body,
            "session:check",
            ResponseLimits::new(body.len() - 1).unwrap()
        )
        .is_err()
    );
    for malformed in [
        b"".as_slice(),
        b"null",
        b"[]",
        b"[false]",
        b"{}{}",
        b"{",
        b"{\"active\":true}\xff",
        b"{\"active\":true,}",
    ] {
        assert!(IntrospectionResponse::parse(200, malformed, expected(150), limits()).is_err());
    }
    let base = serde_json::to_string(&token()).unwrap();
    for depth in [63, 64] {
        let body = format!(
            "{},\"ext\":{}0{}}}",
            &base[..base.len() - 1],
            "[".repeat(depth),
            "]".repeat(depth)
        );
        assert_eq!(
            TokenResponse::parse(200, body.as_bytes(), "session:check", limits()).is_ok(),
            depth == 63
        );
    }
    let error = TokenResponse::parse(
        200,
        b"{\"secret\":\"PRIVATE_SENTINEL\"}",
        "session:check",
        limits(),
    )
    .unwrap_err();
    assert_eq!(format!("{error:?}"), "OAuthResponseRejected");
    assert_eq!(error.to_string(), "OAuth response rejected");
}
