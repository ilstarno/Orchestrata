use super::{AdmissionScope, DatabaseAdmission};

/// Holds one active slot until dropped. Keep it while the protected work remains in flight.
#[must_use = "dropping the permit releases the admission slot"]
pub struct AdmissionPermit {
    pub(super) controller: DatabaseAdmission,
    pub(super) scope: AdmissionScope,
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        self.controller.release(self.scope);
    }
}
