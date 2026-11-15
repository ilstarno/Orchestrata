use std::{collections::VecDeque, sync::Arc};

use super::{
    AdmissionError, AdmissionLimit, AdmissionScope, ScopeLimits, admission_waiter::AdmissionWaiter,
    scope_usage::ScopeUsage,
};

pub(super) struct AdmissionState {
    active_limits: ScopeLimits,
    waiting_limits: Option<ScopeLimits>,
    active: ScopeUsage,
    waiting: ScopeUsage,
    waiters: VecDeque<Arc<AdmissionWaiter>>,
}

impl AdmissionState {
    pub fn new(active_limits: ScopeLimits, waiting_limits: Option<ScopeLimits>) -> Self {
        Self {
            active_limits,
            waiting_limits,
            active: ScopeUsage::default(),
            waiting: ScopeUsage::default(),
            waiters: VecDeque::new(),
        }
    }

    pub fn admit(&mut self, scope: AdmissionScope) -> Result<(), AdmissionLimit> {
        if let Some(limit) = self.active.limit(scope, self.active_limits) {
            return Err(limit);
        }
        self.active.add(scope);
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
        if let Some(limit) = self.waiting.limit(waiter.scope, limits) {
            return Err(AdmissionError::WaitingFull(limit));
        }

        self.waiting.add(waiter.scope);
        self.waiters.push_back(waiter);
        Ok(())
    }

    pub fn release(&mut self, scope: AdmissionScope) -> Option<Arc<AdmissionWaiter>> {
        self.active.release(scope);

        // One release frees at most one slot. Skip scopes that are still at a limit.
        let next = self.waiters.iter().position(|waiter| {
            self.active
                .limit(waiter.scope, self.active_limits)
                .is_none()
        })?;
        let waiter = self.waiters.remove(next)?;
        self.waiting.release(waiter.scope);
        self.active.add(waiter.scope);
        Some(waiter)
    }

    pub fn cancel(&mut self, waiter: &Arc<AdmissionWaiter>) -> Option<Arc<AdmissionWaiter>> {
        if let Some(index) = self
            .waiters
            .iter()
            .position(|pending| Arc::ptr_eq(pending, waiter))
        {
            self.waiters.remove(index);
            self.waiting.release(waiter.scope);
            return None;
        }

        // If already dequeued, this waiter holds an active reservation.
        self.release(waiter.scope)
    }
}
