use orchestrata_core::ids::job_id::JobId;
use uuid::Uuid;

use crate::JobEventKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JobEvent {
    pub event_id: Uuid,
    pub job_id: JobId,
    pub kind: JobEventKind,
}
