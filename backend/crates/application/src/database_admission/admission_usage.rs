use std::collections::{HashMap, hash_map::Entry};

use super::{AdmissionLimit, AdmissionLimits, AdmissionPrincipal};

#[derive(Default)]
pub(super) struct AdmissionUsage {
    global: usize,
    principals: HashMap<AdmissionPrincipal, usize>,
}

impl AdmissionUsage {
    pub fn limit(
        &self,
        principal: AdmissionPrincipal,
        limits: AdmissionLimits,
    ) -> Option<AdmissionLimit> {
        if self.principals.get(&principal).copied().unwrap_or(0) >= limits.per_principal {
            Some(AdmissionLimit::Principal)
        } else if self.global >= limits.global {
            Some(AdmissionLimit::Global)
        } else {
            None
        }
    }

    pub fn add(&mut self, principal: AdmissionPrincipal) {
        self.global += 1;
        *self.principals.entry(principal).or_default() += 1;
    }

    pub fn release(&mut self, principal: AdmissionPrincipal) {
        let Entry::Occupied(mut entry) = self.principals.entry(principal) else {
            unreachable!("each admission must have a matching principal counter");
        };

        self.global -= 1;
        *entry.get_mut() -= 1;
        if *entry.get() == 0 {
            entry.remove();
        }
    }
}

#[cfg(test)]
#[path = "admission_usage_tests.rs"]
mod tests;
