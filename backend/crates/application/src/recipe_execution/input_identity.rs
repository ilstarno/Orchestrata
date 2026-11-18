use serde::Serialize;
use sha2::{Digest, Sha256};

/// SHA-256 of the exact normalized UTF-8 bytes submitted to one Engine run.
/// It is not an entitlement, deduplication key or proof of input provenance.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct InputIdentity(String);

impl InputIdentity {
    pub(super) fn from_records(records: &str) -> Self {
        Self(
            Sha256::digest(records.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
