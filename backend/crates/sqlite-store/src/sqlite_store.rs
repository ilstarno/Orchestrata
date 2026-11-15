use std::{path::Path, time::Duration};

use orchestrata_core::ids::{job_id::JobId, tenant_id::TenantId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::{CancelOutcome, JobEvent, JobEventKind, StoreError, SubmitOutcome};

const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;
const MAX_INPUT_REF_BYTES: usize = 1024;
const MAX_OUTBOX_BATCH: u32 = 1000;
const SCHEMA_VERSION: i64 = 2;
const JOB_SCHEMA: &[&str] = &[
    include_str!("../migrations/001_jobs.sql"),
    include_str!("../migrations/001_job_outbox.sql"),
];
const JOB_LEASE_SCHEMA: &[&str] = &[
    include_str!("../migrations/002_jobs_prepare.sql"),
    include_str!("../migrations/002_job_outbox.sql"),
    include_str!("../migrations/002_jobs_finalize.sql"),
];

pub struct SqliteStore {
    pub(super) connection: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            0 => apply_schema(&transaction, JOB_SCHEMA)?,
            1 | SCHEMA_VERSION => {}
            version => return Err(StoreError::UnsupportedSchema(version)),
        }
        if version < SCHEMA_VERSION {
            apply_schema(&transaction, JOB_LEASE_SCHEMA)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        transaction.commit()?;
        Ok(Self { connection })
    }

    pub fn submit(
        &mut self,
        tenant: TenantId,
        key: &str,
        input_ref: &str,
    ) -> Result<SubmitOutcome, StoreError> {
        if !valid_text(key, MAX_IDEMPOTENCY_KEY_BYTES)
            || !valid_text(input_ref, MAX_INPUT_REF_BYTES)
        {
            return Err(StoreError::InvalidSubmission);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let tenant = tenant.as_uuid().to_string();
        let existing = transaction
            .query_row(
                "SELECT job_id, input_ref FROM jobs WHERE tenant_id = ?1 AND idempotency_key = ?2",
                params![tenant, key],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        if let Some((job_id, stored_input)) = existing {
            let outcome = if stored_input == input_ref {
                SubmitOutcome::Existing(parse_job(&job_id)?)
            } else {
                SubmitOutcome::ConflictingInput
            };
            transaction.commit()?;
            return Ok(outcome);
        }

        let job = JobId::new();
        let job_id = job.as_uuid().to_string();
        transaction.execute(
            "INSERT INTO jobs (tenant_id, job_id, idempotency_key, input_ref, status)
             VALUES (?1, ?2, ?3, ?4, 'pending')",
            params![tenant, job_id, key, input_ref],
        )?;
        enqueue(&transaction, &tenant, &job_id, JobEventKind::Submitted)?;
        transaction.commit()?;
        Ok(SubmitOutcome::Created(job))
    }

    pub fn cancel_pending(
        &mut self,
        tenant: TenantId,
        job: JobId,
    ) -> Result<CancelOutcome, StoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let tenant = tenant.as_uuid().to_string();
        let job = job.as_uuid().to_string();
        let changed = transaction.execute(
            "UPDATE jobs SET status = 'cancelled'
             WHERE tenant_id = ?1 AND job_id = ?2 AND status = 'pending'",
            params![tenant, job],
        )?;
        let outcome = if changed == 1 {
            enqueue(&transaction, &tenant, &job, JobEventKind::Cancelled)?;
            CancelOutcome::Cancelled
        } else {
            let status = transaction
                .query_row(
                    "SELECT status FROM jobs WHERE tenant_id = ?1 AND job_id = ?2",
                    params![tenant, job],
                    |r| r.get::<_, String>(0),
                )
                .optional()?;
            match status.as_deref() {
                Some("cancelled") => CancelOutcome::AlreadyCancelled,
                Some("running" | "completed") => CancelOutcome::NotPending,
                None => CancelOutcome::NotFound,
                Some(_) => return Err(StoreError::InvalidRecord),
            }
        };
        transaction.commit()?;
        Ok(outcome)
    }

    /// Read at most 1000 unpublished events in commit order. Delivery may repeat.
    pub fn pending_events(
        &self,
        tenant: TenantId,
        limit: u32,
    ) -> Result<Vec<JobEvent>, StoreError> {
        let mut sql = self.connection.prepare(
            "SELECT event_id, job_id, kind FROM job_outbox
             WHERE tenant_id = ?1 AND published = 0 ORDER BY sequence LIMIT ?2",
        )?;
        let mut rows = sql.query(params![
            tenant.as_uuid().to_string(),
            limit.min(MAX_OUTBOX_BATCH)
        ])?;
        let mut events = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let job: String = row.get(1)?;
            let kind: String = row.get(2)?;
            events.push(JobEvent {
                event_id: parse_v7(&id)?,
                job_id: parse_job(&job)?,
                kind: JobEventKind::parse(&kind).ok_or(StoreError::InvalidRecord)?,
            });
        }
        Ok(events)
    }

    /// Acknowledge only after successful delivery. Repeated acknowledgements are no-ops.
    pub fn mark_published(&mut self, tenant: TenantId, event: Uuid) -> Result<bool, StoreError> {
        let changed = self.connection.execute(
            "UPDATE job_outbox SET published = 1
             WHERE tenant_id = ?1 AND event_id = ?2 AND published = 0",
            params![tenant.as_uuid().to_string(), event.to_string()],
        )?;
        Ok(changed == 1)
    }
}

fn apply_schema(transaction: &Transaction<'_>, statements: &[&str]) -> Result<(), StoreError> {
    for sql in statements {
        transaction.execute_batch(sql)?;
    }
    Ok(())
}

pub(super) fn valid_text(value: &str, max_bytes: usize) -> bool {
    value.len() <= max_bytes && !value.trim().is_empty() && !value.chars().any(char::is_control)
}

fn parse_v7(value: &str) -> Result<Uuid, StoreError> {
    let id = Uuid::parse_str(value).map_err(|_| StoreError::InvalidRecord)?;
    if id.get_version_num() != 7 {
        return Err(StoreError::InvalidRecord);
    }
    Ok(id)
}

pub(super) fn parse_job(value: &str) -> Result<JobId, StoreError> {
    JobId::parse(parse_v7(value)?).map_err(|_| StoreError::InvalidRecord)
}

pub(super) fn enqueue(
    transaction: &Transaction<'_>,
    tenant: &str,
    job: &str,
    kind: JobEventKind,
) -> Result<(), StoreError> {
    transaction.execute(
        "INSERT INTO job_outbox (tenant_id, event_id, job_id, kind) VALUES (?1, ?2, ?3, ?4)",
        params![tenant, Uuid::now_v7().to_string(), job, kind.as_str()],
    )?;
    Ok(())
}
