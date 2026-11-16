use rusqlite::Transaction;

use crate::StoreError;

const APPLICATION_ID: i64 = 0x4F52_4348;
const SCHEMA_VERSION: i64 = 3;
const JOB_SCHEMA: &[&str] = &[
    include_str!("../migrations/003_jobs.sql"),
    include_str!("../migrations/003_job_outbox.sql"),
];

/// The previous prototype formats do not contain trustworthy local user ownership.
/// Do not rewrite or guess the ownership of existing rows.
pub(super) fn initialize(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    let version = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != 0 && version != SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema(version));
    }
    let marker: i64 = transaction.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if version == SCHEMA_VERSION {
        return if marker == APPLICATION_ID {
            Ok(())
        } else {
            Err(StoreError::IncompatibleDatabase)
        };
    }

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
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}
