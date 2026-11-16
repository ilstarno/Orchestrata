use std::time::Instant;

use openidconnect::{
    AuthUrl, AuthenticationFlow, ClientId, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    RedirectUrl,
    core::{CoreClient, CoreJsonWebKeySet, CoreResponseType},
    url::Url,
};

use super::{
    OidcError, PendingSignIn,
    constants::{MAX_CLIENT_ID_BYTES, MAX_CONFIGURED_URL_BYTES},
};

/// Trusted static OIDC provider configuration. Do not derive it from a request.
pub struct OidcConfig {
    pub(super) issuer: IssuerUrl,
    authorize: AuthUrl,
    pub(super) redirect: RedirectUrl,
    pub(super) client_id: ClientId,
}

impl OidcConfig {
    /// The authorization endpoint must share the issuer's origin in this variant.
    pub fn new(
        issuer: &str,
        authorization_endpoint: &str,
        redirect_uri: &str,
        client_id: &str,
    ) -> Result<Self, OidcError> {
        let issuer_url = configured_https_url(issuer).ok_or(OidcError::InvalidIssuer)?;
        let authorize_url = configured_https_url(authorization_endpoint)
            .filter(|url| url.origin() == issuer_url.origin())
            .ok_or(OidcError::InvalidAuthorizationEndpoint)?;
        configured_https_url(redirect_uri).ok_or(OidcError::InvalidRedirectUri)?;

        if client_id.is_empty()
            || client_id.len() > MAX_CLIENT_ID_BYTES
            || client_id
                .chars()
                .any(|ch| ch.is_control() || ch.is_whitespace())
        {
            return Err(OidcError::InvalidClientId);
        }

        // Keep configured issuer and redirect strings exact for protocol comparisons.
        Ok(Self {
            issuer: IssuerUrl::new(issuer.to_owned()).map_err(|_| OidcError::InvalidIssuer)?,
            authorize: AuthUrl::from_url(authorize_url),
            redirect: RedirectUrl::new(redirect_uri.to_owned())
                .map_err(|_| OidcError::InvalidRedirectUri)?,
            client_id: ClientId::new(client_id.to_owned()),
        })
    }

    /// Returns the public redirect URL and a private, one-time pending transaction.
    ///
    /// Keep the transaction server-side, tied to a trusted browser session
    /// binding. Send only the URL to the browser. Use a monotonic server clock.
    pub fn begin_sign_in(&self, browser_binding: [u8; 32], now: Instant) -> (Url, PendingSignIn) {
        let client = CoreClient::new(
            self.client_id.clone(),
            self.issuer.clone(),
            CoreJsonWebKeySet::default(),
        )
        .set_auth_uri(self.authorize.clone())
        .set_redirect_uri(self.redirect.clone());
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();

        let pending = PendingSignIn {
            state,
            nonce,
            verifier,
            issuer: self.issuer.clone(),
            redirect: self.redirect.clone(),
            client_id: self.client_id.clone(),
            browser_binding,
            started: now,
        };
        (url, pending)
    }
}

pub(super) fn configured_https_url(value: &str) -> Option<Url> {
    let authority = value
        .strip_prefix("https://")?
        .split(['/', '?', '#'])
        .next()?;
    if value.len() > MAX_CONFIGURED_URL_BYTES
        || authority.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace() || ch == '\\')
        || authority.contains(['@', '%'])
    {
        return None;
    }

    let url = Url::parse(value).ok()?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }

    Some(url)
}
