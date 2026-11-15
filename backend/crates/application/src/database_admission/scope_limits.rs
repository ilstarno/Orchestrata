/// A hierarchy of nonzero total, per-tenant, and per-principal limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeLimits {
    pub(super) total: usize,
    pub(super) per_tenant: usize,
    pub(super) per_principal: usize,
}

impl ScopeLimits {
    /// Returns `None` if the limits are zero or the hierarchy is inconsistent.
    pub fn new(total: usize, per_tenant: usize, per_principal: usize) -> Option<Self> {
        if per_principal == 0 || per_principal > per_tenant || per_tenant > total {
            return None;
        }

        Some(Self {
            total,
            per_tenant,
            per_principal,
        })
    }
}
