use std::{collections::VecDeque, sync::Arc};

use super::{
    AdmissionError, AdmissionLimit, AdmissionLimits, AdmissionPrincipal,
    admission_usage::AdmissionUsage, admission_waiter::AdmissionWaiter,
};

pub(super) struct AdmissionState {
    active_limits: AdmissionLimits,
    waiting_limits: Option<AdmissionLimits>,
    active: AdmissionUsage,
    waiting: AdmissionUsage,
    waiters: VecDeque<Arc<AdmissionWaiter>>,
}

impl AdmissionState {
    pub fn new(active_limits: AdmissionLimits, waiting_limits: Option<AdmissionLimits>) -> Self {
        Self {
            active_limits,
            waiting_limits,
            active: AdmissionUsage::default(),
            waiting: AdmissionUsage::default(),
            waiters: VecDeque::new(),
        }
    }

    pub fn admit(&mut self, principal: AdmissionPrincipal) -> Result<(), AdmissionLimit> {
        if let Some(limit) = self.active.limit(principal, self.active_limits) {
            return Err(limit);
        }
        self.active.add(principal);
        Ok(())
    }

    pub fn enqueue(
        &mut self,
        waiter: Arc<AdmissionWaiter>,
        active_limit: AdmissionLimit,
    ) -> Result<(), AdmissionError> {
        let limits = self
            .waiting_limits
            .ok_or(AdmissionError::Busy(active_limit))?;
        if let Some(limit) = self.waiting.limit(waiter.principal, limits) {
            return Err(AdmissionError::WaitingFull(limit));
        }

        self.waiting.add(waiter.principal);
        self.waiters.push_back(waiter);
        Ok(())
    }

    pub fn release(&mut self, principal: AdmissionPrincipal) -> Option<Arc<AdmissionWaiter>> {
        self.active.release(principal);

        // One release frees at most one slot. Skip principals that are still at a limit.
        let next = self.waiters.iter().position(|waiter| {
            self.active
                .limit(waiter.principal, self.active_limits)
                .is_none()
        })?;
        let waiter = self.waiters.remove(next)?;
        self.waiting.release(waiter.principal);
        self.active.add(waiter.principal);
        Some(waiter)
    }

    pub fn cancel(&mut self, waiter: &Arc<AdmissionWaiter>) -> Option<Arc<AdmissionWaiter>> {
        if let Some(index) = self
            .waiters
            .iter()
            .position(|pending| Arc::ptr_eq(pending, waiter))
        {
            self.waiters.remove(index);
            self.waiting.release(waiter.principal);
            return None;
        }

        // If already dequeued, this waiter holds an active reservation.
        self.release(waiter.principal)
    }
}
