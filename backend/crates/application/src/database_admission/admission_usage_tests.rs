use super::*;

#[test]
fn removes_principal_counters_when_the_last_permit_is_released() {
    let mut usage = AdmissionUsage::default();
    for id in 0..10_000 {
        let principal = AdmissionPrincipal::from_trusted_id(id);
        usage.add(principal);
        usage.add(principal);
        usage.release(principal);
        assert_eq!(usage.principals.len(), 1);
        usage.release(principal);
        assert_eq!(usage.global, 0);
        assert!(usage.principals.is_empty());
    }
}
