use std::time::Instant;

use openidconnect::{
    AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl,
};
use subtle::ConstantTimeEq;

use super::{
    BoundAuthorizationCode, OidcError,
    constants::{MAX_AUTHORIZATION_CODE_BYTES, SIGN_IN_LIFETIME},
};

/// Server-side transaction data. Not serializable or suitable for a client cookie.
pub struct PendingSignIn {
    pub(super) state: CsrfToken,
    pub(super) nonce: Nonce,
    pub(super) verifier: PkceCodeVerifier,
    pub(super) issuer: IssuerUrl,
    pub(super) redirect: RedirectUrl,
    pub(super) client_id: ClientId,
    pub(super) browser_binding: [u8; 32],
    pub(super) started: Instant,
}

impl PendingSignIn {
    /// Consumes a pending transaction and binds the callback code to it.
    ///
    /// The caller must atomically consume the stored transaction and pass a
    /// trusted browser session binding, rather than a callback query value.
    /// If present, the provider's callback issuer must exactly match configuration.
    /// This verifies transaction data only; it does not authenticate a subject.
    pub fn accept_callback(
        self,
        browser_binding: &[u8; 32],
        state: &str,
        code: &str,
        response_issuer: Option<&str>,
        now: Instant,
    ) -> Result<BoundAuthorizationCode, OidcError> {
        if now
            .checked_duration_since(self.started)
            .is_none_or(|elapsed| elapsed >= SIGN_IN_LIFETIME)
        {
            return Err(OidcError::ExpiredSignIn);
        }

        let binding_matches = self.browser_binding.ct_eq(browser_binding);
        let state_matches = self.state.secret().as_bytes().ct_eq(state.as_bytes());
        if !bool::from(binding_matches & state_matches)
            || response_issuer.is_some_and(|issuer| issuer != self.issuer.as_str())
            || code.is_empty()
            || code.len() > MAX_AUTHORIZATION_CODE_BYTES
            || code.chars().any(char::is_control)
        {
            return Err(OidcError::InvalidCallback);
        }

        Ok(BoundAuthorizationCode {
            code: AuthorizationCode::new(code.to_owned()),
            pkce_verifier: self.verifier,
            expected_nonce: self.nonce,
            issuer: self.issuer,
            redirect_uri: self.redirect,
            client_id: self.client_id,
        })
    }
}
