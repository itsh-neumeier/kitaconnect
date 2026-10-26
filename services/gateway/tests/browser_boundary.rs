use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hyper::{
    Method, Uri,
    header::{HeaderMap, HeaderValue},
};
use kitaconnect_gateway::{
    BoundaryError, BrowserBoundary, CookieKind, HeaderLimits, Operation, PresentedSecret,
};

fn limits() -> HeaderLimits {
    HeaderLimits {
        count: 16,
        bytes: 4096,
        cookie_bytes: 1024,
    }
}
fn boundary() -> BrowserBoundary {
    BrowserBoundary::new("https://localhost:3443", limits()).unwrap()
}
fn encoded(value: u8) -> String {
    URL_SAFE_NO_PAD.encode([value; 32])
}
fn headers(operation: Operation) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("origin", HeaderValue::from_static("https://localhost:3443"));
    headers.insert("host", HeaderValue::from_static("localhost:3443"));
    if operation != Operation::Context {
        let name = if operation == Operation::LoginStart {
            CookieKind::Context.name()
        } else {
            CookieKind::Session.name()
        };
        headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("theme=light; {name}={}", encoded(7))).unwrap(),
        );
        headers.insert("x-kc-csrf", HeaderValue::from_str(&encoded(8)).unwrap());
    }
    headers
}
fn accepted(operation: Operation, headers: &HeaderMap) -> bool {
    boundary()
        .present(
            operation,
            &Method::POST,
            &operation.path().parse().unwrap(),
            headers,
        )
        .is_ok()
}

#[test]
fn fixed_profiles_keep_secrets_presented_and_separate() {
    for operation in [
        Operation::Context,
        Operation::LoginStart,
        Operation::SessionCheck,
        Operation::Logout,
        Operation::Rotate,
    ] {
        let result = boundary()
            .present(
                operation,
                &Method::POST,
                &operation.path().parse().unwrap(),
                &headers(operation),
            )
            .unwrap();
        assert_eq!(result.operation(), operation);
        assert_eq!(format!("{result:?}"), "PresentedRequest(REDACTED)");
        if operation == Operation::Context {
            assert!(result.credential().is_none());
            assert!(result.csrf().is_none());
        } else {
            assert_eq!(result.credential().unwrap().expose(), &[7; 32]);
            assert_eq!(result.csrf().unwrap().expose(), &[8; 32]);
        }
    }
    let mut request = headers(Operation::LoginStart);
    request.insert(
        "cookie",
        HeaderValue::from_str(&format!("{}={}", CookieKind::Session.name(), encoded(7))).unwrap(),
    );
    assert!(!accepted(Operation::LoginStart, &request));
    let mut request = headers(Operation::SessionCheck);
    request.insert(
        "cookie",
        HeaderValue::from_str(&format!("{}={}", CookieKind::Context.name(), encoded(7))).unwrap(),
    );
    assert!(!accepted(Operation::SessionCheck, &request));
}

#[test]
fn exact_origin_host_and_route_cannot_be_replaced_by_proxy_claims() {
    let operation = Operation::Logout;
    for name in ["origin", "host"] {
        let mut request = headers(operation);
        request.remove(name);
        assert!(!accepted(operation, &request));
        request = headers(operation);
        request.append(name, HeaderValue::from_static("https://localhost:3443"));
        assert!(!accepted(operation, &request));
    }
    for value in [
        "null",
        "https://other.invalid",
        "http://localhost:3443",
        "https://localhost:3443/",
        "https://LOCALHOST:3443",
        "https://localhost:3443 https://other.invalid",
        " https://localhost:3443",
    ] {
        let mut request = headers(operation);
        request.insert("origin", HeaderValue::from_str(value).unwrap());
        request.insert(
            "forwarded",
            HeaderValue::from_static("host=localhost:3443;proto=https"),
        );
        request.insert(
            "x-forwarded-host",
            HeaderValue::from_static("localhost:3443"),
        );
        assert!(!accepted(operation, &request));
    }
    let mut request = headers(operation);
    request.insert("host", HeaderValue::from_static("other.invalid"));
    assert!(!accepted(operation, &request));
    for method in [
        Method::GET,
        Method::HEAD,
        Method::OPTIONS,
        Method::PUT,
        Method::DELETE,
    ] {
        assert!(
            boundary()
                .present(
                    operation,
                    &method,
                    &operation.path().parse().unwrap(),
                    &headers(operation)
                )
                .is_err()
        );
    }
    for uri in [
        "/api/v1/auth/session/logout?x=1",
        "/api/v1/auth/session/logout?",
        "/api/v1/auth/session/check",
        "https://localhost:3443/api/v1/auth/session/logout",
    ] {
        assert!(
            boundary()
                .present(
                    operation,
                    &Method::POST,
                    &uri.parse::<Uri>().unwrap(),
                    &headers(operation)
                )
                .is_err()
        );
    }
    let mut request = headers(operation);
    request.insert(
        "authorization",
        HeaderValue::from_static("Bearer synthetic"),
    );
    assert!(!accepted(operation, &request));
    let mut request = headers(operation);
    request.insert("x-user-id", HeaderValue::from_static("synthetic"));
    request.insert("x-roles", HeaderValue::from_static("admin"));
    let presented = boundary()
        .present(
            operation,
            &Method::POST,
            &operation.path().parse().unwrap(),
            &request,
        )
        .unwrap();
    assert_eq!(presented.credential().unwrap().expose(), &[7; 32]); // No identity/role claim exists on this result.
}

#[test]
fn ambiguous_cookies_headers_and_noncanonical_secrets_are_rejected() {
    let operation = Operation::Rotate;
    for name in ["cookie", "x-kc-csrf"] {
        let mut request = headers(operation);
        request.remove(name);
        assert!(!accepted(operation, &request));
        request = headers(operation);
        let copy = request.get(name).unwrap().clone();
        request.append(name, copy);
        assert!(!accepted(operation, &request));
    }
    for value in [
        String::new(),
        "broken".into(),
        format!(
            "{}={}; {}={}",
            CookieKind::Session.name(),
            encoded(7),
            CookieKind::Session.name(),
            encoded(7)
        ),
        format!(
            "theme=a; theme=b; {}={}",
            CookieKind::Session.name(),
            encoded(7)
        ),
        format!("{}=\"{}\"", CookieKind::Session.name(), encoded(7)),
        format!("{}={};", CookieKind::Session.name(), encoded(7)),
        format!("bad name=a; {}={}", CookieKind::Session.name(), encoded(7)),
        format!("theme=a,b; {}={}", CookieKind::Session.name(), encoded(7)),
    ] {
        let mut request = headers(operation);
        request.insert("cookie", HeaderValue::from_str(&value).unwrap());
        assert!(!accepted(operation, &request));
    }
    for value in [
        "".to_owned(),
        "A".repeat(42),
        "A".repeat(44),
        format!("{}=", encoded(7)),
        "/".repeat(43),
        "A".repeat(42) + "B",
    ] {
        let mut request = headers(operation);
        request.insert("x-kc-csrf", HeaderValue::from_str(&value).unwrap());
        assert!(!accepted(operation, &request));
        request = headers(operation);
        request.insert(
            "cookie",
            HeaderValue::from_str(&format!("{}={value}", CookieKind::Session.name())).unwrap(),
        );
        assert!(!accepted(operation, &request));
    }
    let mut request = headers(Operation::Context);
    request.insert("x-kc-csrf", HeaderValue::from_str(&encoded(8)).unwrap());
    assert!(!accepted(Operation::Context, &request));
}

#[test]
fn explicit_header_and_cookie_work_limits_are_enforced() {
    for bad in [
        HeaderLimits {
            count: 0,
            ..limits()
        },
        HeaderLimits {
            bytes: 0,
            ..limits()
        },
        HeaderLimits {
            cookie_bytes: 0,
            ..limits()
        },
    ] {
        assert!(BrowserBoundary::new("https://localhost:3443", bad).is_err());
    }
    let operation = Operation::SessionCheck;
    let request = headers(operation);
    let uri = operation.path().parse().unwrap();
    assert!(
        BrowserBoundary::new(
            "https://localhost:3443",
            HeaderLimits {
                count: 3,
                ..limits()
            }
        )
        .unwrap()
        .present(operation, &Method::POST, &uri, &request)
        .is_err()
    );
    assert!(
        BrowserBoundary::new(
            "https://localhost:3443",
            HeaderLimits {
                bytes: 64,
                ..limits()
            }
        )
        .unwrap()
        .present(operation, &Method::POST, &uri, &request)
        .is_err()
    );
    assert!(
        BrowserBoundary::new(
            "https://localhost:3443",
            HeaderLimits {
                cookie_bytes: 8,
                ..limits()
            }
        )
        .unwrap()
        .present(operation, &Method::POST, &uri, &request)
        .is_err()
    );
    let mut huge = request;
    huge.insert("x-extra", HeaderValue::from_str(&"a".repeat(4096)).unwrap());
    assert!(!accepted(operation, &huge));
}

#[test]
fn cookie_output_and_secret_diagnostics_preserve_confirmed_attributes() {
    assert!(PresentedSecret::from_bytes(&[1; 31]).is_err());
    assert!(PresentedSecret::from_bytes(&[1; 33]).is_err());
    let secret = PresentedSecret::from_bytes(&[7; 32]).unwrap();
    assert_eq!(format!("{secret:?}"), "PresentedSecret(REDACTED)");
    assert_eq!(format!("{BoundaryError:?}"), "browser request rejected");
    for kind in [CookieKind::Context, CookieKind::Session] {
        let header = kind.issue(&secret).unwrap();
        assert!(header.is_sensitive());
        assert_eq!(
            header.to_str().unwrap(),
            format!(
                "{}={}; Path=/; Secure; HttpOnly; SameSite=Strict",
                kind.name(),
                encoded(7)
            )
        );
        assert!(!format!("{header:?}").contains(&encoded(7)));
        let clear = kind.clear();
        assert!(clear.is_sensitive());
        assert_eq!(
            clear.to_str().unwrap(),
            format!(
                "{}=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0",
                kind.name()
            )
        );
    }
    for origin in [
        "http://localhost:3000",
        "https://localhost:3443/",
        "https://localhost:3443/a",
        "https://u@localhost:3443",
        "https://localhost:3443?x=1",
        "https://localhost:3443#x",
        "https://localhost:443",
        "null",
    ] {
        assert!(
            BrowserBoundary::new(origin, limits()).is_err(),
            "accepted {origin}"
        );
    }
}
