use tokio::sync::Notify;

use super::AdmissionPrincipal;

pub(super) struct AdmissionWaiter {
    pub principal: AdmissionPrincipal,
    pub ready: Notify,
}
