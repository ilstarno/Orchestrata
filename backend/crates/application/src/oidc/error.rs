#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OidcError {
    InvalidIssuer,
    InvalidAuthorizationEndpoint,
    InvalidRedirectUri,
    InvalidClientId,
    ExpiredSignIn,
    InvalidCallback,
}
