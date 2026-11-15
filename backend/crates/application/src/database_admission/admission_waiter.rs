use tokio::sync::Notify;

use super::AdmissionScope;

pub(super) struct AdmissionWaiter {
    pub scope: AdmissionScope,
    pub ready: Notify,
}
