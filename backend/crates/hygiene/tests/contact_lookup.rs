use orchestrata_hygiene::{
    CanonicalContact, ContactLookupError, ContactLookupPurpose, ContactLookupScope,
    contact_lookup_digest,
};

const SYNTHETIC_KEY: [u8; 32] = [0x5c; 32];
const SYNTHETIC_TENANT: &[u8] = b"synthetic-tenant-a";
const EMAIL: CanonicalContact<'_> = CanonicalContact::Email("alice@example.invalid");
const PHONE: CanonicalContact<'_> = CanonicalContact::Phone("+12025550101");

#[test]
fn fixed_vector_matches_independent_hmac_sha256_with_v1_framing() {
    let expected = [
        0x6a, 0x36, 0x9d, 0x01, 0xdf, 0x59, 0xed, 0x47, 0x57, 0x25, 0xec, 0x94, 0x21, 0x0d, 0x7a,
        0xb2, 0x5c, 0x8b, 0x25, 0xf5, 0xec, 0x15, 0x77, 0xf6, 0xd2, 0xf4, 0x39, 0x3d, 0x0c, 0x66,
        0x21, 0xdf,
    ];
    let tenant = directory(SYNTHETIC_TENANT);

    let digest = contact_lookup_digest(&SYNTHETIC_KEY, tenant, EMAIL).unwrap();

    assert_eq!(digest.as_bytes(), &expected);
}

#[test]
fn tenant_purpose_kind_and_key_domains_do_not_share_digests() {
    let scope = directory(SYNTHETIC_TENANT);
    let phone_as_email = CanonicalContact::Email("+12025550101");
    let expected = contact_lookup_digest(&SYNTHETIC_KEY, scope, PHONE).unwrap();

    let cases = [
        (&SYNTHETIC_KEY, directory(b"synthetic-tenant-b"), PHONE),
        (
            &SYNTHETIC_KEY,
            ContactLookupScope {
                purpose: ContactLookupPurpose::DuplicateDetection,
                ..scope
            },
            PHONE,
        ),
        (&SYNTHETIC_KEY, scope, phone_as_email),
        (&[0xa3; 32], scope, PHONE),
    ];
    for (key, isolated_scope, contact) in cases {
        let actual = contact_lookup_digest(key, isolated_scope, contact).unwrap();
        assert_ne!(actual.as_bytes(), expected.as_bytes());
    }
    assert_eq!(
        contact_lookup_digest(&SYNTHETIC_KEY, scope, PHONE)
            .unwrap()
            .as_bytes(),
        expected.as_bytes()
    );
}

#[test]
fn invalid_inputs_are_rejected_without_a_digest() {
    let scope = directory(SYNTHETIC_TENANT);
    assert!(matches!(
        contact_lookup_digest(&SYNTHETIC_KEY[..31], scope, EMAIL),
        Err(ContactLookupError::InvalidKeyLength)
    ));
    for tenant in [b"".as_slice(), &[0; 257]] {
        assert!(matches!(
            contact_lookup_digest(&SYNTHETIC_KEY, directory(tenant), EMAIL),
            Err(ContactLookupError::InvalidTenantScope)
        ));
    }
    for invalid in ["", " alice@example.invalid", "alice\n@example.invalid"] {
        assert!(matches!(
            contact_lookup_digest(&SYNTHETIC_KEY, scope, CanonicalContact::Email(invalid)),
            Err(ContactLookupError::InvalidContact)
        ));
    }
    assert!(matches!(
        contact_lookup_digest(
            &SYNTHETIC_KEY,
            scope,
            CanonicalContact::Phone(&"x".repeat(1025)),
        ),
        Err(ContactLookupError::InvalidContact)
    ));
}

fn directory(tenant: &[u8]) -> ContactLookupScope<'_> {
    ContactLookupScope {
        tenant,
        purpose: ContactLookupPurpose::DirectoryLookup,
    }
}
