use orchestrata_hygiene::inspect;
use orchestrata_solver::{Error, Problem, Solution, solve};

use crate::solve_error::SolveError;

const MAX_EVALUATIONS: u64 = 100_000;

pub struct SolveService;

impl SolveService {
    pub fn execute(problem: &Problem) -> Result<Option<Solution>, SolveError> {
        inspect(problem).map_err(|_| SolveError::InvalidInput)?;
        solve(problem, MAX_EVALUATIONS).map_err(|error| match error {
            Error::EmptyDomain(_) | Error::InvalidVariable(_) | Error::InvalidWeight => {
                SolveError::InvalidInput
            }
            Error::TooManyAssignments => SolveError::SearchLimitExceeded,
            Error::ScoreOverflow | Error::InvalidSolution => SolveError::ExecutionFailed,
        })
    }

    pub async fn execute_async(problem: Problem) -> Result<Option<Solution>, SolveError> {
        tokio::task::spawn_blocking(move || Self::execute(&problem))
            .await
            .map_err(|_| SolveError::ExecutionFailed)?
    }
}
