use super::{AdmissionPrincipal, DatabaseAdmission};

/// Holds one active slot until dropped. Keep it while the protected work remains in flight.
#[must_use = "dropping the permit releases the admission slot"]
pub struct AdmissionPermit {
    pub(super) controller: DatabaseAdmission,
    pub(super) principal: AdmissionPrincipal,
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        self.controller.release(self.principal);
    }
}
