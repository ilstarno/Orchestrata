#[path = "enums/module.rs"]
pub mod enums;
mod reference_solver;

pub use enums::{constraint::Constraint, error::Error};
pub use reference_solver::solve;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Variable {
    pub name: String,
    pub domain: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Score {
    pub hard: i64,
    pub soft: i64,
}

impl Score {
    fn better_than(self, other: Self) -> bool {
        (self.hard, self.soft) > (other.hard, other.soft)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Solution {
    pub values: Vec<i64>,
    pub score: Score,
    pub evaluated: u64,
}

pub fn validate(problem: &Problem) -> Result<(), Error> {
    for (index, variable) in problem.variables.iter().enumerate() {
        if variable.domain.is_empty() { return Err(Error::EmptyDomain(index)); }
    }
    for constraint in &problem.constraints {
        let (weight, indices): (i64, &[usize]) = match constraint {
            Constraint::AllDifferent { variables, hard_weight } => (*hard_weight, variables),
            Constraint::NotEqualOffset { left, right, hard_weight, .. } => {
                if *hard_weight <= 0 {
                    return Err(Error::InvalidWeight);
                }
                if *left >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*left));
                }
                if *right >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*right));
                }
                continue;
            }
            Constraint::PreferValue { variable, soft_weight, .. } => {
                if *soft_weight <= 0 {
                    return Err(Error::InvalidWeight);
                }
                if *variable >= problem.variables.len() {
                    return Err(Error::InvalidVariable(*variable));
                }
                continue;
            }
        };
        if weight <= 0 {
            return Err(Error::InvalidWeight);
        }
        for &index in indices {
            if index >= problem.variables.len() {
                return Err(Error::InvalidVariable(index));
            }
        }
    }
    Ok(())
}

pub fn score(problem: &Problem, values: &[i64]) -> Result<Score, Error> {
    validate(problem)?;
    if values.len() != problem.variables.len() {
        return Err(Error::InvalidVariable(values.len()));
    }
    for (index, (variable, value)) in problem.variables.iter().zip(values).enumerate() {
        if !variable.domain.contains(value) { return Err(Error::InvalidVariable(index)); }
    }
    let mut result = Score { hard: 0, soft: 0 };
    for constraint in &problem.constraints {
        match constraint {
            Constraint::AllDifferent { variables, hard_weight } => {
                for i in 0..variables.len() {
                    for j in i + 1..variables.len() {
                        if values[variables[i]] == values[variables[j]] {
                            result.hard = result.hard.checked_sub(*hard_weight).ok_or(Error::ScoreOverflow)?;
                        }
                    }
                }
            }
            Constraint::NotEqualOffset { left, right, offset, hard_weight } => {
                if i128::from(values[*left]) - i128::from(values[*right]) == i128::from(*offset) {
                    result.hard = result.hard.checked_sub(*hard_weight).ok_or(Error::ScoreOverflow)?;
                }
            }
            Constraint::PreferValue { variable, value, soft_weight } => {
                if values[*variable] != *value {
                    result.soft = result.soft.checked_sub(*soft_weight).ok_or(Error::ScoreOverflow)?;
                }
            }
        }
    }
    Ok(result)
}

