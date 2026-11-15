use orchestrata_hygiene::*;
use orchestrata_solver::{Problem, Variable};

#[test]
fn duplicate_domains_are_rejected() {
    let problem = Problem {
        variables: vec![Variable {
            name: "task".into(),
            domain: vec![1, 1],
        }],
        constraints: vec![],
    };
    assert_eq!(
        inspect(&problem),
        Err(HygieneError::DuplicateDomainValue(0))
    );
}

#[test]
fn duplicate_names_are_rejected() {
    let problem = Problem {
        variables: vec![
            Variable {
                name: "task".into(),
                domain: vec![1],
            },
            Variable {
                name: "task".into(),
                domain: vec![2],
            },
        ],
        constraints: vec![],
    };
    assert_eq!(
        inspect(&problem),
        Err(HygieneError::DuplicateVariableName("task".into()))
    );
}
