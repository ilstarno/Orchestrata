/// A digest of an opaque session token, computed by the trusted caller before storage.
/// This is not the cookie value and deliberately has no Debug or Display representation.
pub struct SessionDigest([u8; 32]);

impl SessionDigest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
