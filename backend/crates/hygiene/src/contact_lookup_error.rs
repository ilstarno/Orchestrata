#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactLookupError {
    InvalidKeyLength,
    InvalidTenantScope,
    InvalidContact,
}
