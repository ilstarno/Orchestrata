use orchestrata_hygiene::inspect;
use orchestrata_solver::{Error, Problem, Solution, solve, solve_heuristic, validate};

use crate::solve_error::SolveError;

const MAX_EVALUATIONS: u64 = 100_000;

pub struct SolveService;

impl SolveService {
    pub fn execute(problem: &Problem) -> Result<Option<Solution>, SolveError> {
        inspect(problem).map_err(|_| SolveError::InvalidInput)?;
        validate(problem).map_err(|_| SolveError::InvalidInput)?;
        match solve(problem, MAX_EVALUATIONS) {
            Ok(solution) => Ok(solution),
            Err(Error::TooManyAssignments) => {
                solve_heuristic(problem, MAX_EVALUATIONS).map_err(|_| SolveError::ExecutionFailed)
            }
            Err(_) => Err(SolveError::ExecutionFailed),
        }
    }

    pub async fn execute_async(problem: Problem) -> Result<Option<Solution>, SolveError> {
        tokio::task::spawn_blocking(move || Self::execute(&problem))
            .await
            .map_err(|_| SolveError::ExecutionFailed)?
    }
}
