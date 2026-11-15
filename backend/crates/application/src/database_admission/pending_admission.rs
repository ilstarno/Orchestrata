use std::sync::Arc;

use super::{AdmissionPermit, DatabaseAdmission, admission_waiter::AdmissionWaiter};

pub(super) struct PendingAdmission {
    pub controller: DatabaseAdmission,
    pub waiter: Option<Arc<AdmissionWaiter>>,
}

impl PendingAdmission {
    pub async fn wait(&self) {
        self.waiter
            .as_ref()
            .expect("pending admission has a waiter")
            .ready
            .notified()
            .await;
    }

    pub fn into_permit(mut self) -> AdmissionPermit {
        let waiter = self.waiter.take().expect("granted admission has a waiter");
        AdmissionPermit {
            controller: self.controller.clone(),
            scope: waiter.scope,
        }
    }
}

impl Drop for PendingAdmission {
    fn drop(&mut self) {
        if let Some(waiter) = &self.waiter {
            self.controller.cancel(waiter);
        }
    }
}
