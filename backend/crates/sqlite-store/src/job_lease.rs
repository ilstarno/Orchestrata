use orchestrata_core::ids::{job_id::JobId, tenant_id::TenantId};
use uuid::Uuid;

pub struct JobLease {
    pub(super) tenant_id: TenantId,
    pub(super) job_id: JobId,
    pub(super) lease_id: Uuid,
    pub(super) expires_at_unix_ms: i64,
    pub(super) input_ref: String,
}

impl JobLease {
    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn input_ref(&self) -> &str {
        &self.input_ref
    }

    pub fn expires_at_unix_ms(&self) -> i64 {
        self.expires_at_unix_ms
    }
}
