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
mod tests {
    use std::sync::mpsc;

    use tokio::sync::oneshot;

    use super::*;

    #[tokio::test]
    async fn rejects_saturation_without_running_submitted_work() {
        let admission = SolveAdmission::new(1);
        let worker = admission.clone();
        let (started, running) = oneshot::channel();
        let (release, blocked) = mpsc::channel();
        let task = tokio::spawn(async move {
            worker
                .execute(move || {
                    started.send(()).unwrap();
                    let _ = blocked.recv();
                    Ok(None)
                })
                .await
        });

        running.await.unwrap();
        assert!(matches!(
            admission.execute(|| panic!("overloaded work ran")).await,
            Err(ApiError::SolveBusy)
        ));
        release.send(()).unwrap();
        assert!(matches!(task.await.unwrap(), Ok(None)));
        assert!(matches!(admission.execute(|| Ok(None)).await, Ok(None)));
    }

    #[tokio::test]
    async fn aborting_request_does_not_release_slot_while_blocking_work_runs() {
        let admission = SolveAdmission::new(1);
        let worker = admission.clone();
        let (started, running) = oneshot::channel();
        let (release, blocked) = mpsc::channel();
        let task = tokio::spawn(async move {
            worker
                .execute(move || {
                    started.send(()).unwrap();
                    let _ = blocked.recv();
                    Ok(None)
                })
                .await
        });

        running.await.unwrap();
        task.abort();
        assert!(matches!(task.await, Err(error) if error.is_cancelled()));
        assert!(matches!(
            admission
                .execute(|| panic!("cancelled request freed a running slot"))
                .await,
            Err(ApiError::SolveBusy)
        ));
        release.send(()).unwrap();
    }
}
