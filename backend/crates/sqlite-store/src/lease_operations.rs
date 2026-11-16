use std::time::Duration;

use orchestrata_core::ids::job_id::JobId;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::{
    CompleteOutcome, JobEventKind, JobLease, JobOwnerId, SqliteStore, StoreError,
    sqlite_store::{enqueue, parse_job, valid_text},
};

const MAX_LEASE_DURATION: Duration = Duration::from_secs(86_400);
const MAX_RESULT_REF_BYTES: usize = 1024;

impl SqliteStore {
    /// Claims a pending or expired job across the local installation for a trusted worker.
    pub fn claim_next(&mut self, duration: Duration) -> Result<Option<JobLease>, StoreError> {
        let duration_ms = lease_millis(duration)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = database_time_ms(&transaction)?;
        let expires_at_unix_ms = now
            .checked_add(duration_ms)
            .ok_or(StoreError::InvalidLeaseDuration)?;
        let lease_id = Uuid::now_v7();
        let row = transaction
            .query_row(
                "UPDATE jobs SET status = 'running', lease_id = ?3, lease_expires_ms = ?2
                 WHERE job_id = (
                    SELECT job_id FROM jobs WHERE status IN ('pending', 'running')
                      AND (status = 'pending' OR lease_expires_ms <= ?1)
                    ORDER BY job_id LIMIT 1
                 )
                 RETURNING job_id, owner_id, input_ref",
                params![now, expires_at_unix_ms, lease_id.to_string()],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        let lease = row
            .map(|(job, owner, input_ref)| -> Result<_, StoreError> {
                let owner = Uuid::parse_str(&owner).map_err(|_| StoreError::InvalidRecord)?;
                Ok(JobLease {
                    owner_id: JobOwnerId::parse(owner).ok_or(StoreError::InvalidRecord)?,
                    job_id: parse_job(&job)?,
                    lease_id,
                    expires_at_unix_ms,
                    input_ref,
                })
            })
            .transpose()?;
        transaction.commit()?;
        Ok(lease)
    }

    /// Commits the result reference and completion event only for the current, unexpired lease.
    pub fn complete(
        &mut self,
        lease: &JobLease,
        result_ref: &str,
    ) -> Result<CompleteOutcome, StoreError> {
        if !valid_text(result_ref, MAX_RESULT_REF_BYTES) {
            return Err(StoreError::InvalidResultReference);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = database_time_ms(&transaction)?;
        let owner = lease.owner_id.as_uuid().to_string();
        let job = lease.job_id.as_uuid().to_string();
        let changed = transaction.execute(
            "UPDATE jobs SET status = 'completed', result_ref = ?5,
                    lease_id = NULL, lease_expires_ms = NULL
             WHERE owner_id = ?1 AND job_id = ?2 AND status = 'running'
               AND lease_id = ?3 AND lease_expires_ms > ?4",
            params![owner, job, lease.lease_id.to_string(), now, result_ref],
        )?;
        if changed != 1 {
            return Ok(CompleteOutcome::LeaseNotCurrent);
        }

        enqueue(&transaction, &job, JobEventKind::Completed)?;
        transaction.commit()?;
        Ok(CompleteOutcome::Completed)
    }

    /// Resolves a committed reference, including for a job named by a completion event.
    pub fn completed_result(
        &self,
        owner: JobOwnerId,
        job: JobId,
    ) -> Result<Option<String>, StoreError> {
        Ok(self
            .connection
            .query_row(
                "SELECT result_ref FROM jobs
                 WHERE owner_id = ?1 AND job_id = ?2 AND status = 'completed'",
                params![owner.as_uuid().to_string(), job.as_uuid().to_string()],
                |r| r.get(0),
            )
            .optional()?)
    }
}

fn database_time_ms(transaction: &Transaction<'_>) -> Result<i64, StoreError> {
    Ok(transaction.query_row(
        "SELECT CAST(ROUND(unixepoch('subsec') * 1000) AS INTEGER)",
        [],
        |r| r.get(0),
    )?)
}

fn lease_millis(duration: Duration) -> Result<i64, StoreError> {
    if duration < Duration::from_millis(1) || duration > MAX_LEASE_DURATION {
        return Err(StoreError::InvalidLeaseDuration);
    }
    i64::try_from(duration.as_millis()).map_err(|_| StoreError::InvalidLeaseDuration)
}
