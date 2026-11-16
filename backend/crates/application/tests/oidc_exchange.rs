#![cfg(feature = "oidc")]

use std::{collections::BTreeMap, io, time::Instant};

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{Duration, Utc};
use openidconnect::{
    Audience, EmptyAdditionalClaims, HttpRequest, HttpResponse, IssuerUrl, JsonWebKeyId, Nonce,
    PrivateSigningKey, StandardClaims, SubjectIdentifier,
    core::{
        CoreEdDsaPrivateSigningKey, CoreIdToken, CoreIdTokenClaims, CoreJsonWebKeySet,
        CoreJwsSigningAlgorithm,
    },
    http::{Method, StatusCode},
    url::form_urlencoded,
};
use orchestrata_application::oidc::{
    BoundAuthorizationCode, OidcConfig, OidcError, OidcTokenExchange,
};

const ISSUER: &str = "https://identity.example.invalid/oidc";
const AUTHORIZE: &str = "https://identity.example.invalid/oidc/authorize";
const TOKEN: &str = "https://identity.example.invalid/oidc/token";
const REDIRECT: &str = "https://app.example.invalid/login/oidc/callback";
const CLIENT: &str = "example-orchestrata";
const BROWSER: [u8; 32] = [42; 32];

fn config(client: &str) -> OidcConfig {
    OidcConfig::new(ISSUER, AUTHORIZE, REDIRECT, client).unwrap()
}

fn callback(config: &OidcConfig) -> BoundAuthorizationCode {
    let now = Instant::now();
    let (url, pending) = config.begin_sign_in(BROWSER, now);
    let state = url.query_pairs().find(|(key, _)| key == "state").unwrap().1;
    pending
        .accept_callback(&BROWSER, &state, "synthetic-code", Some(ISSUER), now)
        .unwrap()
}

fn synthetic_key(seed: u8) -> CoreEdDsaPrivateSigningKey {
    // PKCS#8 for a fixed, deliberately public Ed25519 test seed.
    let mut der = vec![
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20,
    ];
    der.extend([seed; 32]);
    let pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        STANDARD.encode(der),
    );
    CoreEdDsaPrivateSigningKey::from_ed25519_pem(&pem, Some(JsonWebKeyId::new("test-key".into())))
        .unwrap()
}

fn sign(
    key: &CoreEdDsaPrivateSigningKey,
    issuer: &str,
    audience: &str,
    nonce: Option<Nonce>,
    expired: bool,
    subject: &str,
) -> CoreIdToken {
    let now = Utc::now();
    let issued = now - Duration::minutes(10);
    let expiration = if expired {
        now - Duration::minutes(1)
    } else {
        now + Duration::minutes(5)
    };
    let claims = CoreIdTokenClaims::new(
        IssuerUrl::new(issuer.into()).unwrap(),
        vec![Audience::new(audience.into())],
        expiration,
        issued,
        StandardClaims::new(SubjectIdentifier::new(subject.into())),
        EmptyAdditionalClaims {},
    )
    .set_nonce(nonce);
    CoreIdToken::new(claims, key, CoreJwsSigningAlgorithm::EdDsa, None, None).unwrap()
}

fn response(token: Option<&CoreIdToken>) -> HttpResponse {
    let mut json = serde_json::json!({
        "access_token": "synthetic-access-token",
        "token_type": "Bearer",
    });
    if let Some(token) = token {
        json["id_token"] = serde_json::to_value(token).unwrap();
    }

    openidconnect::http::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&json).unwrap())
        .unwrap()
}

#[test]
fn consumes_pkce_code_and_returns_only_stable_verified_identity() {
    let config = config(CLIENT);
    let bound = callback(&config);
    let expected_verifier = bound.pkce_verifier().secret().clone();
    let key = synthetic_key(1);
    let token = sign(
        &key,
        ISSUER,
        CLIENT,
        Some(bound.expected_nonce().clone()),
        false,
        "synthetic-subject",
    );
    let exchange = OidcTokenExchange::new(
        &config,
        TOKEN,
        CoreJsonWebKeySet::new(vec![key.as_verification_key()]),
        None,
    )
    .unwrap();

    let identity = exchange
        .exchange(bound, &|request: HttpRequest| {
            assert_eq!(request.uri().to_string(), TOKEN);
            assert_eq!(request.method(), Method::POST);
            let fields: BTreeMap<_, _> = form_urlencoded::parse(request.body()).collect();
            assert_eq!(fields["grant_type"], "authorization_code");
            assert_eq!(fields["code"], "synthetic-code");
            assert_eq!(fields["redirect_uri"], REDIRECT);
            assert_eq!(fields["client_id"], CLIENT);
            assert!(fields["code_verifier"] == expected_verifier);
            assert!(!fields.contains_key("nonce"));
            Ok::<_, io::Error>(response(Some(&token)))
        })
        .unwrap();

    assert_eq!(identity.issuer(), ISSUER);
    assert_eq!(identity.subject(), "synthetic-subject");
}

#[test]
fn rejects_bad_signatures_and_untrusted_claims() {
    let config = config(CLIENT);
    let key = synthetic_key(1);
    let bad_key = synthetic_key(2);
    let exchange = OidcTokenExchange::new(
        &config,
        TOKEN,
        CoreJsonWebKeySet::new(vec![key.as_verification_key()]),
        None,
    )
    .unwrap();
    for case in [
        "signature",
        "issuer",
        "audience",
        "nonce",
        "missing-nonce",
        "expiry",
        "subject",
    ] {
        let bound = callback(&config);
        let token = sign(
            if case == "signature" { &bad_key } else { &key },
            if case == "issuer" {
                "https://different.example.invalid/oidc"
            } else {
                ISSUER
            },
            if case == "audience" {
                "different-client"
            } else {
                CLIENT
            },
            match case {
                "nonce" => Some(Nonce::new("another-nonce".into())),
                "missing-nonce" => None,
                _ => Some(bound.expected_nonce().clone()),
            },
            case == "expiry",
            if case == "subject" {
                ""
            } else {
                "synthetic-subject"
            },
        );
        assert_eq!(
            exchange.exchange(bound, &|_: HttpRequest| Ok::<_, io::Error>(response(Some(
                &token
            )))),
            Err(OidcError::InvalidIdToken),
            "{case}",
        );
    }
}

#[test]
fn rejects_other_client_configuration_before_transport_and_requires_bounded_trusted_keys() {
    let trusted_config = config(CLIENT);
    let other = config("other-client");
    let keys = CoreJsonWebKeySet::new(vec![synthetic_key(1).as_verification_key()]);
    for endpoint in [
        "http://identity.example.invalid/oidc/token",
        "https://different.example.invalid/oidc/token",
        "https://identity.example.invalid/oidc/token?forward=other",
    ] {
        assert_eq!(
            OidcTokenExchange::new(&trusted_config, endpoint, keys.clone(), None).err(),
            Some(OidcError::InvalidTokenEndpoint)
        );
    }
    assert_eq!(
        OidcTokenExchange::new(&trusted_config, TOKEN, CoreJsonWebKeySet::default(), None).err(),
        Some(OidcError::InvalidVerificationKeys),
    );

    let exchange = OidcTokenExchange::new(&trusted_config, TOKEN, keys, None).unwrap();
    assert_eq!(
        exchange.exchange(
            callback(&other),
            &|_: HttpRequest| -> Result<HttpResponse, io::Error> {
                panic!("wrong-client callbacks must not reach transport");
            }
        ),
        Err(OidcError::WrongTransaction),
    );
}

#[test]
fn rejects_missing_id_token_and_oversized_or_redirected_response() {
    let config = config(CLIENT);
    let keys = CoreJsonWebKeySet::new(vec![synthetic_key(1).as_verification_key()]);
    let exchange = OidcTokenExchange::new(&config, TOKEN, keys, None).unwrap();
    assert_eq!(
        exchange.exchange(callback(&config), &|_: HttpRequest| Ok::<_, io::Error>(
            response(None)
        )),
        Err(OidcError::MissingIdToken),
    );

    for status in [StatusCode::OK, StatusCode::FOUND] {
        let transport = |_: HttpRequest| {
            let body = if status == StatusCode::OK {
                vec![b'x'; 64 * 1024 + 1]
            } else {
                vec![]
            };
            Ok::<_, io::Error>(
                openidconnect::http::Response::builder()
                    .status(status)
                    .header("Location", "https://different.example.invalid/oidc/token")
                    .body(body)
                    .unwrap(),
            )
        };
        assert_eq!(
            exchange.exchange(callback(&config), &transport),
            Err(OidcError::TokenExchangeFailed),
        );
    }
}
