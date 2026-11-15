use orchestrata_solver::{Problem, Solution, solve};

use crate::{
    constants::SOLVER_MAX_EVALUATIONS,
    enums::solve_service_error::SolveServiceError,
};

pub struct SolveService;

impl SolveService {
    pub async fn execute(problem: Problem) -> Result<Option<Solution>, SolveServiceError> {
        tokio::task::spawn_blocking(move || solve(&problem, SOLVER_MAX_EVALUATIONS))
            .await
            .map_err(|_| SolveServiceError::ExecutionFailed)?
            .map_err(|_| SolveServiceError::InvalidProblem)
    }
}
