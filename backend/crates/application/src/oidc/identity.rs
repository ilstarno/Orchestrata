/// Stable identifier from a verified ID token. This is not a local session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcIdentity {
    issuer: String,
    subject: String,
}

impl OidcIdentity {
    pub(super) fn new(issuer: String, subject: String) -> Self {
        Self { issuer, subject }
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}
