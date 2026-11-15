use orchestrata_solver::{Problem, Solution, solve};

use crate::enums::solve_service_error::SolveServiceError;

pub struct SolveService;

impl SolveService {
    pub async fn execute(problem: Problem) -> Result<Option<Solution>, SolveServiceError> {
        tokio::task::spawn_blocking(move || solve(&problem, 100_000))
            .await
            .map_err(|_| SolveServiceError::ExecutionFailed)?
            .map_err(|_| SolveServiceError::InvalidProblem)
    }
}
