use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{sync::Notify, time};

use super::{
    AdmissionError, AdmissionPermit, AdmissionScope, ScopeLimits, admission_state::AdmissionState,
    admission_waiter::AdmissionWaiter, pending_admission::PendingAdmission,
};

/// Process-local admission control ahead of database pool checkout.
#[derive(Clone)]
pub struct DatabaseAdmission {
    state: Arc<Mutex<AdmissionState>>,
}

impl DatabaseAdmission {
    /// Passing no waiting limits disables queueing.
    pub fn new(active_limits: ScopeLimits, waiting_limits: Option<ScopeLimits>) -> Self {
        Self {
            state: Arc::new(Mutex::new(AdmissionState::new(
                active_limits,
                waiting_limits,
            ))),
        }
    }

    /// Returns immediately without queueing or waiting for capacity.
    pub fn try_acquire(&self, scope: AdmissionScope) -> Result<AdmissionPermit, AdmissionError> {
        let mut state = self.state.lock().map_err(|_| AdmissionError::Unavailable)?;
        state.admit(scope).map_err(AdmissionError::Busy)?;
        Ok(AdmissionPermit {
            controller: self.clone(),
            scope,
        })
    }

    /// Waits for capacity up to the caller's timeout, using a Tokio timer.
    ///
    /// Queued callers are selected in arrival order among currently eligible
    /// scopes. Dropping the future cancels a waiter even if it was just granted.
    /// A zero timeout never enqueues.
    pub async fn acquire_timeout(
        &self,
        scope: AdmissionScope,
        timeout: Duration,
    ) -> Result<AdmissionPermit, AdmissionError> {
        let pending = {
            let mut state = self.state.lock().map_err(|_| AdmissionError::Unavailable)?;
            let active_limit = match state.admit(scope) {
                Ok(()) => {
                    return Ok(AdmissionPermit {
                        controller: self.clone(),
                        scope,
                    });
                }
                Err(limit) => limit,
            };

            if timeout.is_zero() {
                return Err(AdmissionError::Busy(active_limit));
            }

            let waiter = Arc::new(AdmissionWaiter {
                scope,
                ready: Notify::new(),
            });
            state.enqueue(waiter.clone(), active_limit)?;
            PendingAdmission {
                controller: self.clone(),
                waiter: Some(waiter),
            }
        };

        time::timeout(timeout, pending.wait())
            .await
            .map_err(|_| AdmissionError::TimedOut)?;
        Ok(pending.into_permit())
    }

    pub(super) fn release(&self, scope: AdmissionScope) {
        let granted = self
            .state
            .lock()
            .ok()
            .and_then(|mut state| state.release(scope));
        if let Some(waiter) = granted {
            waiter.ready.notify_one();
        }
    }

    pub(super) fn cancel(&self, waiter: &Arc<AdmissionWaiter>) {
        let granted = self
            .state
            .lock()
            .ok()
            .and_then(|mut state| state.cancel(waiter));
        if let Some(waiter) = granted {
            waiter.ready.notify_one();
        }
    }
}
