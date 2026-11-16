use std::io;

use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, RedirectUrl, SyncHttpClient, TokenResponse, TokenUrl,
    core::{CoreClient, CoreJsonWebKeySet, CoreJwsSigningAlgorithm},
};

use super::{
    BoundAuthorizationCode, OidcConfig, OidcError, OidcIdentity,
    config::configured_https_url,
    constants::{MAX_SUBJECT_BYTES, MAX_TOKEN_RESPONSE_BYTES, MAX_VERIFICATION_KEYS},
};

/// Connects one configured OIDC client to a trusted token endpoint and verification keys.
pub struct OidcTokenExchange {
    issuer: IssuerUrl,
    client_id: ClientId,
    redirect_uri: RedirectUrl,
    token_endpoint: TokenUrl,
    keys: CoreJsonWebKeySet,
    secret: Option<ClientSecret>,
}

impl OidcTokenExchange {
    /// The endpoint must be HTTPS and share the issuer's origin.
    ///
    /// Supply only verification keys trusted for this issuer. This constructor
    /// does not discover providers or fetch keys.
    pub fn new(
        config: &OidcConfig,
        token_endpoint: &str,
        keys: CoreJsonWebKeySet,
        secret: Option<ClientSecret>,
    ) -> Result<Self, OidcError> {
        let endpoint = configured_https_url(token_endpoint)
            .filter(|url| url.origin() == config.issuer.url().origin())
            .ok_or(OidcError::InvalidTokenEndpoint)?;
        if keys.keys().is_empty() || keys.keys().len() > MAX_VERIFICATION_KEYS {
            return Err(OidcError::InvalidVerificationKeys);
        }

        Ok(Self {
            issuer: config.issuer.clone(),
            client_id: config.client_id.clone(),
            redirect_uri: config.redirect.clone(),
            token_endpoint: TokenUrl::from_url(endpoint),
            keys,
            secret,
        })
    }

    /// Consumes the bound code, exchanges it, and verifies the signed ID token.
    ///
    /// The injected transport is trusted: it must validate TLS and the server
    /// name, disable redirects, impose deadlines and a streaming response limit,
    /// and avoid retaining or logging protocol secrets. This call may block.
    /// It returns no access or refresh token and creates no local session.
    pub fn exchange(
        &self,
        bound: BoundAuthorizationCode,
        transport: &impl SyncHttpClient,
    ) -> Result<OidcIdentity, OidcError> {
        if bound.issuer != self.issuer
            || bound.client_id != self.client_id
            || bound.redirect_uri != self.redirect_uri
        {
            return Err(OidcError::WrongTransaction);
        }

        let client = CoreClient::new(
            self.client_id.clone(),
            self.issuer.clone(),
            self.keys.clone(),
        )
        .set_token_uri(self.token_endpoint.clone())
        .set_redirect_uri(self.redirect_uri.clone());
        let client = if let Some(secret) = &self.secret {
            client.set_client_secret(secret.clone())
        } else {
            client
        };

        let response = client
            .exchange_code(bound.code)
            .set_pkce_verifier(bound.pkce_verifier)
            .request(&|request| {
                let response = transport
                    .call(request)
                    .map_err(|_| io::Error::other("token transport failed"))?;
                if response.body().len() > MAX_TOKEN_RESPONSE_BYTES {
                    return Err(io::Error::other("token response is too large"));
                }
                Ok(response)
            })
            .map_err(|_| OidcError::TokenExchangeFailed)?;

        let id_token = response.id_token().ok_or(OidcError::MissingIdToken)?;
        let verifier = client.id_token_verifier().set_allowed_algs([
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            CoreJwsSigningAlgorithm::EcdsaP256Sha256,
            CoreJwsSigningAlgorithm::EdDsa,
        ]);
        let claims = id_token
            .claims(&verifier, &bound.expected_nonce)
            .map_err(|_| OidcError::InvalidIdToken)?;
        let subject = claims.subject().as_str();
        if subject.is_empty()
            || subject.len() > MAX_SUBJECT_BYTES
            || !subject.is_ascii()
            || subject.bytes().any(|byte| byte.is_ascii_control())
        {
            return Err(OidcError::InvalidIdToken);
        }

        Ok(OidcIdentity::new(
            self.issuer.to_string(),
            subject.to_owned(),
        ))
    }
}
