use orchestrata_ffi::module::abi::{OrcConstraint, OrcScore, OrcVariable};
use orchestrata_ffi::{OrcStatus, orc_abi_version, orc_solve};

#[test]
fn version_is_stable() {
    assert_eq!(orc_abi_version(), 1);
}

#[test]
fn solves_typed_assignments() {
    let domain = [0_i64, 1];
    let variables = [OrcVariable { domain: domain.as_ptr(), domain_len: domain.len() }];
    let constraints = [OrcConstraint {
        kind: 2, left: 0, right: 0, value: 1, weight: 1,
    }];
    let mut output = [0_i64];
    let mut score = OrcScore { hard: -1, soft: -1 };
    let status = unsafe {
        orc_solve(variables.as_ptr(), 1, constraints.as_ptr(), 1, 10, output.as_mut_ptr(), 1, &mut score)
    };
    assert_eq!(status, OrcStatus::Ok);
    assert_eq!(output, [1]);
    assert_eq!((score.hard, score.soft), (0, 0));
}

#[test]
fn rejects_null_arguments() {
    let mut score = OrcScore { hard: 0, soft: 0 };
    let status = unsafe {
        orc_solve(std::ptr::null(), 1, std::ptr::null(), 0, 10, std::ptr::null_mut(), 0, &mut score)
    };
    assert_eq!(status, OrcStatus::InvalidArgument);
}
