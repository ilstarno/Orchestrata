use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::{CanonicalContact, ContactDigest, ContactLookupError, ContactLookupScope};

const DOMAIN: &[u8] = b"orchestrata.contact-lookup.v1";
const MIN_KEY_BYTES: usize = 32;
const MAX_TENANT_BYTES: usize = 256;
const MAX_CONTACT_BYTES: usize = 1024;

/// Derives an equality lookup value from a high-entropy, lookup-specific secret of at least
/// 32 bytes. The function borrows the secret and contact and neither stores nor logs them.
/// Use a stable, trusted scope and consistent canonicalization at the caller.
pub fn contact_lookup_digest(
    key: &[u8],
    scope: ContactLookupScope<'_>,
    contact: CanonicalContact<'_>,
) -> Result<ContactDigest, ContactLookupError> {
    if key.len() < MIN_KEY_BYTES {
        return Err(ContactLookupError::InvalidKeyLength);
    }
    if scope.tenant.is_empty() || scope.tenant.len() > MAX_TENANT_BYTES {
        return Err(ContactLookupError::InvalidTenantScope);
    }
    let (kind, value) = contact.fields();
    if value.is_empty()
        || value.len() > MAX_CONTACT_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(ContactLookupError::InvalidContact);
    }

    let mut mac =
        Hmac::<Sha256>::new_from_slice(key).map_err(|_| ContactLookupError::InvalidKeyLength)?;
    for field in [
        DOMAIN,
        scope.purpose.tag(),
        scope.tenant,
        kind,
        value.as_bytes(),
    ] {
        // All fields are bounded well below the 32-bit framing limit.
        mac.update(&(field.len() as u32).to_be_bytes());
        mac.update(field);
    }
    Ok(ContactDigest(mac.finalize().into_bytes().into()))
}
