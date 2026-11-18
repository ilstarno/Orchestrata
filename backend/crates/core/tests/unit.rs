use orchestrata_core::{errors::CoreError, ids::job_id::JobId, score::HardMediumSoftScore};
use uuid::Uuid;

#[test]
fn generated_ids_use_version_seven() {
    assert_eq!(JobId::new().as_uuid().get_version_num(), 7);
}

#[test]
fn other_uuid_versions_are_rejected() {
    assert_eq!(JobId::parse(Uuid::nil()), Err(CoreError::InvalidIdentifier));
}

#[test]
fn score_order_is_lexicographic() {
    let feasible = HardMediumSoftScore {
        hard: 0,
        medium: -100,
        soft: -100,
    };
    let infeasible = HardMediumSoftScore {
        hard: -1,
        medium: 100,
        soft: 100,
    };
    assert!(feasible > infeasible);
    assert!(feasible.is_feasible());
    assert!(!infeasible.is_feasible());
}

#[test]
fn score_overflow_is_reported() {
    let a = HardMediumSoftScore {
        hard: i64::MAX,
        medium: 0,
        soft: 0,
    };
    let b = HardMediumSoftScore {
        hard: 1,
        medium: 0,
        soft: 0,
    };
    assert_eq!(a.checked_add(b), Err(CoreError::ScoreOverflow));
}
