use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{sync::Notify, time};

use super::{
    AdmissionError, AdmissionLimits, AdmissionPermit, AdmissionPrincipal,
    admission_state::AdmissionState, admission_waiter::AdmissionWaiter,
    pending_admission::PendingAdmission,
};

/// Process-local admission control ahead of database pool checkout.
#[derive(Clone)]
pub struct DatabaseAdmission {
    state: Arc<Mutex<AdmissionState>>,
}

impl DatabaseAdmission {
    /// Passing no waiting limits disables queueing.
    pub fn new(active_limits: AdmissionLimits, waiting_limits: Option<AdmissionLimits>) -> Self {
        Self {
            state: Arc::new(Mutex::new(AdmissionState::new(
                active_limits,
                waiting_limits,
            ))),
        }
    }

    /// Attempts admission without queueing or waiting for capacity.
    pub fn try_acquire(
        &self,
        principal: AdmissionPrincipal,
    ) -> Result<AdmissionPermit, AdmissionError> {
        let mut state = self.state.lock().map_err(|_| AdmissionError::Unavailable)?;
        state.admit(principal).map_err(AdmissionError::Busy)?;
        Ok(AdmissionPermit {
            controller: self.clone(),
            principal,
        })
    }

    /// Waits for capacity up to the caller's timeout, using a Tokio timer.
    ///
    /// Queued callers are selected in arrival order among currently eligible
    /// principals. Dropping the future cancels a waiter even if it was just granted.
    /// A zero timeout never enqueues.
    pub async fn acquire_timeout(
        &self,
        principal: AdmissionPrincipal,
        timeout: Duration,
    ) -> Result<AdmissionPermit, AdmissionError> {
        let pending = {
            let mut state = self.state.lock().map_err(|_| AdmissionError::Unavailable)?;
            let active_limit = match state.admit(principal) {
                Ok(()) => {
                    return Ok(AdmissionPermit {
                        controller: self.clone(),
                        principal,
                    });
                }
                Err(limit) => limit,
            };

            if timeout.is_zero() {
                return Err(AdmissionError::Busy(active_limit));
            }

            let waiter = Arc::new(AdmissionWaiter {
                principal,
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

    pub(super) fn release(&self, principal: AdmissionPrincipal) {
        let granted = self
            .state
            .lock()
            .ok()
            .and_then(|mut state| state.release(principal));
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
