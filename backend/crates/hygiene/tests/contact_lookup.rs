use orchestrata_hygiene::{
    CanonicalContact, ContactLookupError, ContactLookupPurpose, contact_lookup_digest,
};

const SYNTHETIC_KEY: [u8; 32] = [0x5c; 32];
const PURPOSE: ContactLookupPurpose = ContactLookupPurpose::DirectoryLookup;
const EMAIL: CanonicalContact<'_> = CanonicalContact::Email("alice@example.invalid");
const PHONE: CanonicalContact<'_> = CanonicalContact::Phone("+12025550101");

#[test]
fn fixed_vector_matches_independent_hmac_sha256_with_v2_framing() {
    let expected = [
        0xff, 0xd1, 0x97, 0x54, 0x8e, 0xe1, 0x2a, 0xec, 0x43, 0xc4, 0xeb, 0xf2, 0x52, 0xae, 0xd4,
        0x30, 0xad, 0xe3, 0x82, 0x9f, 0xe0, 0x99, 0x40, 0xdb, 0xbe, 0x43, 0x1f, 0x1b, 0x7d, 0x67,
        0x4c, 0x67,
    ];
    let digest = contact_lookup_digest(&SYNTHETIC_KEY, PURPOSE, EMAIL).unwrap();
    assert_eq!(digest.as_bytes(), &expected);
}

#[test]
fn key_purpose_and_contact_kind_each_separate_digests() {
    let baseline = contact_lookup_digest(&SYNTHETIC_KEY, PURPOSE, PHONE).unwrap();
    let another_kind = CanonicalContact::Email("+12025550101");
    for (key, purpose, contact) in [
        (&[0xa3; 32], PURPOSE, PHONE),
        (
            &SYNTHETIC_KEY,
            ContactLookupPurpose::DuplicateDetection,
            PHONE,
        ),
        (&SYNTHETIC_KEY, PURPOSE, another_kind),
    ] {
        let alternate = contact_lookup_digest(key, purpose, contact).unwrap();
        assert_ne!(alternate.as_bytes(), baseline.as_bytes());
    }
    let repeated = contact_lookup_digest(&SYNTHETIC_KEY, PURPOSE, PHONE).unwrap();
    assert_eq!(repeated.as_bytes(), baseline.as_bytes());
}

#[test]
fn invalid_inputs_are_rejected_without_a_digest() {
    assert!(matches!(
        contact_lookup_digest(&SYNTHETIC_KEY[..31], PURPOSE, EMAIL),
        Err(ContactLookupError::InvalidKeyLength)
    ));
    for invalid in ["", " alice@example.invalid", "alice\n@example.invalid"] {
        assert!(matches!(
            contact_lookup_digest(&SYNTHETIC_KEY, PURPOSE, CanonicalContact::Email(invalid)),
            Err(ContactLookupError::InvalidContact)
        ));
    }
    assert!(matches!(
        contact_lookup_digest(
            &SYNTHETIC_KEY,
            PURPOSE,
            CanonicalContact::Phone(&"x".repeat(1025))
        ),
        Err(ContactLookupError::InvalidContact)
    ));
}
