/// Nonzero global and per-principal admission limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionLimits {
    pub(super) global: usize,
    pub(super) per_principal: usize,
}

impl AdmissionLimits {
    /// Returns `None` if either limit is zero or the principal limit exceeds the global limit.
    pub fn new(global: usize, per_principal: usize) -> Option<Self> {
        if per_principal == 0 || per_principal > global {
            return None;
        }

        Some(Self {
            global,
            per_principal,
        })
    }
}
