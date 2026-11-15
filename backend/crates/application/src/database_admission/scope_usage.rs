use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use super::{AdmissionLimit, AdmissionScope, ScopeLimits};

#[derive(Default)]
pub(super) struct ScopeUsage {
    total: usize,
    tenants: HashMap<u128, usize>,
    principals: HashMap<AdmissionScope, usize>,
}

impl ScopeUsage {
    pub fn limit(&self, scope: AdmissionScope, limits: ScopeLimits) -> Option<AdmissionLimit> {
        if self.principals.get(&scope).copied().unwrap_or(0) >= limits.per_principal {
            Some(AdmissionLimit::Principal)
        } else if self.tenants.get(&scope.tenant).copied().unwrap_or(0) >= limits.per_tenant {
            Some(AdmissionLimit::Tenant)
        } else if self.total >= limits.total {
            Some(AdmissionLimit::Total)
        } else {
            None
        }
    }

    pub fn add(&mut self, scope: AdmissionScope) {
        self.total += 1;
        *self.tenants.entry(scope.tenant).or_default() += 1;
        *self.principals.entry(scope).or_default() += 1;
    }

    pub fn release(&mut self, scope: AdmissionScope) {
        self.total -= 1;
        decrement(&mut self.tenants, scope.tenant);
        decrement(&mut self.principals, scope);
    }
}

fn decrement<K: Eq + Hash>(counts: &mut HashMap<K, usize>, key: K) {
    let Entry::Occupied(mut entry) = counts.entry(key) else {
        unreachable!("each admission must have a matching scope counter");
    };
    *entry.get_mut() -= 1;
    if *entry.get() == 0 {
        entry.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_keys_are_removed_after_every_last_release() {
        let mut usage = ScopeUsage::default();
        for id in 0..10_000 {
            let scope = AdmissionScope {
                tenant: id,
                principal: id,
            };
            usage.add(scope);
            usage.add(scope);
            usage.release(scope);
            assert_eq!(usage.tenants.len(), 1);
            assert_eq!(usage.principals.len(), 1);
            usage.release(scope);
            assert_eq!(usage.total, 0);
            assert!(usage.tenants.is_empty());
            assert!(usage.principals.is_empty());
        }
    }
}
