use std::sync::Arc;

use orchestrata_application::{solve_error::SolveError, solve_service::SolveService};
use orchestrata_solver::{Problem, Solution};
use tokio::sync::Semaphore;

use crate::api::error::ApiError;

#[derive(Clone)]
pub struct SolveAdmission {
    permits: Arc<Semaphore>,
}

impl SolveAdmission {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn solve(&self, problem: Problem) -> Result<Option<Solution>, ApiError> {
        self.execute(move || SolveService::execute(&problem)).await
    }

    async fn execute(
        &self,
        operation: impl FnOnce() -> Result<Option<Solution>, SolveError> + Send + 'static,
    ) -> Result<Option<Solution>, ApiError> {
        let permit = Arc::clone(&self.permits)
            .try_acquire_owned()
            .map_err(|_| ApiError::SolveBusy)?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            operation()
        })
        .await
        .map_err(|_| SolveError::ExecutionFailed)?
        .map_err(Into::into)
    }
}

#[cfg(test)]
#[path = "../../../../../backend_tests/api/solve_admission_unit.rs"]
mod tests;
