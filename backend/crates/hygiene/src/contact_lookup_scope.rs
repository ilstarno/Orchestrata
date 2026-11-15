use crate::ContactLookupPurpose;

#[derive(Clone, Copy)]
pub struct ContactLookupScope<'a> {
    /// A stable identifier from the caller's trusted tenant context, not a request parameter.
    pub tenant: &'a [u8],
    pub purpose: ContactLookupPurpose,
}
