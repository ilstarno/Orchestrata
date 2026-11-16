use openidconnect::{AuthorizationCode, ClientId, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl};

/// One transaction's code-exchange inputs, not a user identity or session.
/// Exchange only through the trusted issuer's token endpoint; verify the ID
/// token (including the expected nonce) before treating any subject as signed in.
pub struct BoundAuthorizationCode {
    pub(super) code: AuthorizationCode,
    pub(super) pkce_verifier: PkceCodeVerifier,
    pub(super) expected_nonce: Nonce,
    pub(super) issuer: IssuerUrl,
    pub(super) redirect_uri: RedirectUrl,
    pub(super) client_id: ClientId,
}

impl BoundAuthorizationCode {
    pub fn authorization_code(&self) -> &AuthorizationCode {
        &self.code
    }

    pub fn pkce_verifier(&self) -> &PkceCodeVerifier {
        &self.pkce_verifier
    }

    pub fn expected_nonce(&self) -> &Nonce {
        &self.expected_nonce
    }

    pub fn issuer(&self) -> &IssuerUrl {
        &self.issuer
    }

    pub fn redirect_uri(&self) -> &RedirectUrl {
        &self.redirect_uri
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }
}
