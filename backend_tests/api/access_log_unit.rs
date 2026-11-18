use super::*;

#[test]
fn unrecognized_method_tokens_are_not_logged() {
    assert_eq!(known_method(&Method::POST), "POST");
    assert_eq!(
        known_method(&Method::from_bytes(b"UNTRUSTED-TOKEN").unwrap()),
        "OTHER"
    );
}
