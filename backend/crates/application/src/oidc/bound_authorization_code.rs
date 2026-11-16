use openidconnect::{AuthorizationCode, ClientId, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl};

/// One transaction's code-exchange inputs, not a user identity or session.
/// Exchange only through the trusted issuer's token endpoint; verify the ID
/// token (including the expected nonce) before treating any subject as signed in.
pub struct BoundAuthorizationCode {
    pub code: AuthorizationCode,
    pub pkce_verifier: PkceCodeVerifier,
    pub expected_nonce: Nonce,
    pub issuer: IssuerUrl,
    pub redirect_uri: RedirectUrl,
    pub client_id: ClientId,
}
