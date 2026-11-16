use rusqlite::Transaction;

use crate::StoreError;

const APPLICATION_ID: i64 = 0x4F52_4348;
const JOB_SCHEMA_VERSION: i64 = 3;
const SCHEMA_VERSION: i64 = 4;
const JOB_SCHEMA: &[&str] = &[
    include_str!("../migrations/003_jobs.sql"),
    include_str!("../migrations/003_job_outbox.sql"),
];
const MAPPING_SCHEMA: &str = include_str!("../migrations/004_column_mappings.sql");

/// The earlier prototype formats lack trustworthy local user ownership. Do not guess it.
pub(super) fn initialize(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    let version = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if !matches!(version, 0 | JOB_SCHEMA_VERSION | SCHEMA_VERSION) {
        return Err(StoreError::UnsupportedSchema(version));
    }
    let marker: i64 = transaction.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if version == 0 {
        initialize_jobs(transaction, marker)?;
    } else if marker != APPLICATION_ID {
        return Err(StoreError::IncompatibleDatabase);
    } else {
        verify_job_schema(transaction)?;
        if version == SCHEMA_VERSION {
            transaction
                .prepare("SELECT mapping_id, owner_id, mapping_json FROM column_mappings LIMIT 0")
                .map_err(|_| StoreError::IncompatibleDatabase)?;
            return Ok(());
        }
    }

    transaction.execute_batch(MAPPING_SCHEMA)?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

fn initialize_jobs(transaction: &Transaction<'_>, marker: i64) -> Result<(), StoreError> {
    let populated: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name NOT GLOB 'sqlite_*')",
        [],
        |row| row.get(0),
    )?;
    if marker != 0 || populated {
        return Err(StoreError::IncompatibleDatabase);
    }
    for sql in JOB_SCHEMA {
        transaction.execute_batch(sql)?;
    }
    transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
    Ok(())
}

fn verify_job_schema(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction
        .prepare("SELECT job_id, owner_id, idempotency_key, input_ref, status, lease_id, lease_expires_ms, result_ref FROM jobs LIMIT 0")
        .map_err(|_| StoreError::IncompatibleDatabase)?;
    transaction
        .prepare("SELECT event_id, job_id, kind, published, sequence FROM job_outbox LIMIT 0")
        .map_err(|_| StoreError::IncompatibleDatabase)?;
    Ok(())
}
