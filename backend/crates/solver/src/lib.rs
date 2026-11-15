//! A generic finite-domain assignment solver with a reference scorer.
//! The search is exhaustive; it is intended for small correctness fixtures.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub domain: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    AllDifferent { variables: Vec<usize>, hard_weight: i64 },
    NotEqualOffset { left: usize, right: usize, offset: i64, hard_weight: i64 },
    PreferValue { variable: usize, value: i64, soft_weight: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problem {
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub hard: i64,
    pub soft: i64,
}

impl Score {
    fn better_than(self, other: Self) -> bool {
        (self.hard, self.soft) > (other.hard, other.soft)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Solution {
    pub values: Vec<i64>,
    pub score: Score,
    pub evaluated: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyDomain(usize),
    InvalidVariable(usize),
    InvalidWeight,
    TooManyAssignments,
}

pub fn validate(problem: &Problem) -> Result<(), Error> {
    for (index, variable) in problem.variables.iter().enumerate() {
        if variable.domain.is_empty() { return Err(Error::EmptyDomain(index)); }
    }
    for constraint in &problem.constraints {
        let (indices, weight): (Vec<usize>, i64) = match constraint {
            Constraint::AllDifferent { variables, hard_weight } => (variables.clone(), *hard_weight),
            Constraint::NotEqualOffset { left, right, hard_weight, .. } => (vec![*left, *right], *hard_weight),
            Constraint::PreferValue { variable, soft_weight, .. } => (vec![*variable], *soft_weight),
        };
        if weight <= 0 { return Err(Error::InvalidWeight); }
        for index in indices {
            if index >= problem.variables.len() { return Err(Error::InvalidVariable(index)); }
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
                            result.hard = result.hard.saturating_sub(*hard_weight);
                        }
                    }
                }
            }
            Constraint::NotEqualOffset { left, right, offset, hard_weight } => {
                if i128::from(values[*left]) - i128::from(values[*right]) == i128::from(*offset) {
                    result.hard = result.hard.saturating_sub(*hard_weight);
                }
            }
            Constraint::PreferValue { variable, value, soft_weight } => {
                if values[*variable] != *value {
                    result.soft = result.soft.saturating_sub(*soft_weight);
                }
            }
        }
    }
    Ok(result)
}

pub fn solve(problem: &Problem, max_evaluations: u64) -> Result<Option<Solution>, Error> {
    validate(problem)?;
    let mut combinations = 1u64;
    for variable in &problem.variables {
        combinations = combinations.checked_mul(variable.domain.len() as u64)
            .ok_or(Error::TooManyAssignments)?;
    }
    if combinations > max_evaluations { return Err(Error::TooManyAssignments); }
    let mut values = vec![0; problem.variables.len()];
    let mut best: Option<Solution> = None;
    let mut evaluated = 0u64;
    fn visit(
        index: usize, problem: &Problem, values: &mut [i64],
        best: &mut Option<Solution>, evaluated: &mut u64,
    ) -> Result<(), Error> {
        if index == values.len() {
            *evaluated += 1;
            let current = score(problem, values)?;
            if best.as_ref().is_none_or(|old| current.better_than(old.score)) {
                *best = Some(Solution {
                    values: values.to_vec(), score: current, evaluated: *evaluated,
                });
            }
            return Ok(());
        }
        for &value in &problem.variables[index].domain {
            values[index] = value;
            visit(index + 1, problem, values, best, evaluated)?;
        }
        Ok(())
    }
    visit(0, problem, &mut values, &mut best, &mut evaluated)?;
    if let Some(solution) = &mut best { solution.evaluated = evaluated; }
    Ok(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queens(n: usize) -> Problem {
        let variables = (0..n).map(|i| Variable {
            name: format!("queen_{i}"), domain: (0..n as i64).collect(),
        }).collect();
        let mut constraints = vec![Constraint::AllDifferent {
            variables: (0..n).collect(), hard_weight: 1,
        }];
        for i in 0..n {
            for j in i + 1..n {
                let delta = j as i64 - i as i64;
                constraints.push(Constraint::NotEqualOffset {
                    left: i, right: j, offset: delta, hard_weight: 1,
                });
                constraints.push(Constraint::NotEqualOffset {
                    left: i, right: j, offset: -delta, hard_weight: 1,
                });
            }
        }
        Problem { variables, constraints }
    }

    #[test]
    fn four_queens_has_a_feasible_solution() {
        let problem = queens(4);
        let solution = solve(&problem, 256).unwrap().unwrap();
        assert_eq!(solution.score.hard, 0);
        assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
        assert_eq!(solution.evaluated, 256);
    }

    #[test]
    fn impossible_three_queens_is_reported_infeasible() {
        let solution = solve(&queens(3), 27).unwrap().unwrap();
        assert!(solution.score.hard < 0);
    }

    #[test]
    fn assignment_prefers_soft_objective_after_feasibility() {
        let problem = Problem {
            variables: vec![
                Variable { name: "a".into(), domain: vec![0, 1] },
                Variable { name: "b".into(), domain: vec![0, 1] },
            ],
            constraints: vec![
                Constraint::AllDifferent { variables: vec![0, 1], hard_weight: 1 },
                Constraint::PreferValue { variable: 0, value: 1, soft_weight: 3 },
            ],
        };
        let solution = solve(&problem, 4).unwrap().unwrap();
        assert_eq!(solution.values, vec![1, 0]);
        assert_eq!(solution.score, Score { hard: 0, soft: 0 });
    }

    #[test]
    fn invalid_reference_is_rejected() {
        let problem = Problem {
            variables: vec![Variable { name: "a".into(), domain: vec![1] }],
            constraints: vec![Constraint::PreferValue {
                variable: 5, value: 1, soft_weight: 1,
            }],
        };
        assert_eq!(validate(&problem), Err(Error::InvalidVariable(5)));
    }
}
