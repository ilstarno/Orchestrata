use std::collections::HashSet;

use orchestrata_solver::{Constraint, Problem};

use crate::HygieneError;

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
