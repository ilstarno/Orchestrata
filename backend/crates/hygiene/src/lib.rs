use std::collections::HashSet;

use orchestrata_solver::{Constraint, Problem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HygieneError {
    BlankVariableName(usize),
    DuplicateVariableName(String),
    DuplicateDomainValue(usize),
    DuplicateConstraintVariable,
    EmptyConstraint,
}

pub fn inspect(problem: &Problem) -> Result<(), HygieneError> {
    let mut names = HashSet::new();

    for (index, variable) in problem.variables.iter().enumerate() {
        let name = variable.name.trim();
        if name.is_empty() {
            return Err(HygieneError::BlankVariableName(index));
        }
        if !names.insert(name) {
            return Err(HygieneError::DuplicateVariableName(name.to_owned()));
        }

        let unique_values: HashSet<_> = variable.domain.iter().collect();
        if unique_values.len() != variable.domain.len() {
            return Err(HygieneError::DuplicateDomainValue(index));
        }
    }

    for constraint in &problem.constraints {
        let Constraint::AllDifferent { variables, .. } = constraint else {
            continue;
        };

        if variables.is_empty() {
            return Err(HygieneError::EmptyConstraint);
        }

        let unique_variables: HashSet<_> = variables.iter().collect();
        if unique_variables.len() != variables.len() {
            return Err(HygieneError::DuplicateConstraintVariable);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use orchestrata_solver::Variable;

    #[test]
    fn duplicate_domains_are_rejected() {
        let problem = Problem {
            variables: vec![Variable { name: "task".into(), domain: vec![1, 1] }],
            constraints: vec![],
        };
        assert_eq!(inspect(&problem), Err(HygieneError::DuplicateDomainValue(0)));
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let problem = Problem {
            variables: vec![
                Variable { name: "task".into(), domain: vec![1] },
                Variable { name: "task".into(), domain: vec![2] },
            ],
            constraints: vec![],
        };
        assert_eq!(
            inspect(&problem),
            Err(HygieneError::DuplicateVariableName("task".into()))
        );
    }
}
