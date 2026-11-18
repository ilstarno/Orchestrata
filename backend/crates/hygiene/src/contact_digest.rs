/// A deterministic, pseudonymous contact lookup value. It is not an anonymization guarantee.
pub struct ContactDigest(pub(super) [u8; 32]);

impl ContactDigest {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
