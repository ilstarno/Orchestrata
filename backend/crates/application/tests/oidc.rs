#![cfg(feature = "oidc")]

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use openidconnect::{PkceCodeChallenge, url::Url};
use orchestrata_application::oidc::{OidcConfig, OidcError, PendingSignIn};

const ISSUER: &str = "https://identity.example.invalid/oidc";
const AUTHORIZE: &str = "https://identity.example.invalid/oidc/authorize";
const REDIRECT: &str = "https://app.example.invalid/login/oidc/callback";
const CLIENT_ID: &str = "example-orchestrata";
const BROWSER: [u8; 32] = [57; 32];

fn begin(now: Instant) -> (Url, PendingSignIn) {
    OidcConfig::new(ISSUER, AUTHORIZE, REDIRECT, CLIENT_ID)
        .unwrap()
        .begin_sign_in(BROWSER, now)
}

fn query(url: &Url) -> HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

#[test]
fn authorization_and_callback_are_bound_by_s256_state_and_nonce() {
    let now = Instant::now();
    let (authorization_url, pending) = begin(now);
    let mut endpoint = authorization_url.clone();
    endpoint.set_query(None);
    assert_eq!(endpoint.as_str(), AUTHORIZE);

    let params = query(&authorization_url);
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["client_id"], CLIENT_ID);
    assert_eq!(params["redirect_uri"], REDIRECT);
    assert_eq!(params["scope"], "openid");
    assert_eq!(params["code_challenge_method"], "S256");
    assert!(!params.contains_key("code_verifier"));

    let accepted = pending
        .accept_callback(
            &BROWSER,
            &params["state"],
            "synthetic-code",
            Some(ISSUER),
            now + Duration::from_secs(1),
        )
        .unwrap();

    assert_eq!(accepted.code.secret(), "synthetic-code");
    assert_eq!(accepted.expected_nonce.secret(), &params["nonce"]);
    assert_eq!(accepted.issuer.as_str(), ISSUER);
    assert_eq!(accepted.redirect_uri.as_str(), REDIRECT);
    assert_eq!(accepted.client_id.as_str(), CLIENT_ID);
    let challenge = PkceCodeChallenge::from_code_verifier_sha256(&accepted.pkce_verifier);
    assert_eq!(challenge.as_str(), params["code_challenge"]);
}

#[test]
fn transactions_do_not_reuse_protocol_values() {
    let now = Instant::now();
    let first = query(&begin(now).0);
    let second = query(&begin(now).0);
    for field in ["state", "nonce", "code_challenge"] {
        assert_ne!(first[field], second[field]);
    }
}

#[test]
fn rejects_wrong_browser_state_issuer_and_empty_or_controlled_code() {
    let now = Instant::now();
    let wrong_browser = [0; 32];
    let oversized = "x".repeat(4097);
    for (browser, wrong_state, issuer, code) in [
        (&wrong_browser, false, Some(ISSUER), "code"),
        (&BROWSER, true, Some(ISSUER), "code"),
        (
            &BROWSER,
            false,
            Some("https://identity.example.invalid/oidc/"),
            "code",
        ),
        (
            &BROWSER,
            false,
            Some("https://unrelated.example.invalid/oidc"),
            "code",
        ),
        (&BROWSER, false, None, ""),
        (&BROWSER, false, None, "code\n"),
        (&BROWSER, false, None, oversized.as_str()),
    ] {
        let (url, pending) = begin(now);
        let params = query(&url);
        let state = if wrong_state {
            "wrong-state"
        } else {
            &params["state"]
        };
        assert_eq!(
            pending
                .accept_callback(browser, state, code, issuer, now)
                .err(),
            Some(OidcError::InvalidCallback)
        );
    }
}

#[test]
fn expires_exactly_at_five_minutes() {
    let now = Instant::now();
    let (url, pending) = begin(now);
    assert!(
        pending
            .accept_callback(
                &BROWSER,
                &query(&url)["state"],
                "code",
                None,
                now + Duration::from_secs(299),
            )
            .is_ok()
    );

    for callback_time in [now + Duration::from_secs(300), now - Duration::from_secs(1)] {
        let (url, pending) = begin(now);
        assert_eq!(
            pending
                .accept_callback(&BROWSER, &query(&url)["state"], "code", None, callback_time)
                .err(),
            Some(OidcError::ExpiredSignIn)
        );
    }
}

#[test]
fn rejects_untrusted_configuration() {
    for issuer in [
        "http://identity.example.invalid",
        "https:///identity.example.invalid",
        "https://identity.example.invalid?issuer=other",
        "https://username@identity.example.invalid",
        "https://@identity.example.invalid",
        "https://identity.example.invalid#fragment",
    ] {
        assert_eq!(
            OidcConfig::new(issuer, AUTHORIZE, REDIRECT, CLIENT_ID).err(),
            Some(OidcError::InvalidIssuer)
        );
    }

    for authorize in [
        "http://identity.example.invalid:80/authorize",
        "https://identity.example.invalid:8443/authorize",
        "https://different.example.invalid/authorize",
        "https://identity.example.invalid/authorize?redirect_uri=other",
    ] {
        assert_eq!(
            OidcConfig::new(ISSUER, authorize, REDIRECT, CLIENT_ID).err(),
            Some(OidcError::InvalidAuthorizationEndpoint)
        );
    }

    for redirect in [
        "/login/oidc/callback",
        "http://app.example.invalid/login/oidc/callback",
        "https://example.invalid/login/oidc/callback#fragment",
        "https://app.example.invalid/login/oidc/callback?destination=other",
    ] {
        assert_eq!(
            OidcConfig::new(ISSUER, AUTHORIZE, redirect, CLIENT_ID).err(),
            Some(OidcError::InvalidRedirectUri)
        );
    }

    assert_eq!(
        OidcConfig::new(ISSUER, AUTHORIZE, REDIRECT, "").err(),
        Some(OidcError::InvalidClientId)
    );
}
