use std::{fs, path::PathBuf};

use orchestrata_sqlite_store::{JobOwnerId, SqliteStore, StoreError, SubmitOutcome};
use rusqlite::{Connection, params};
use uuid::Uuid;

const V1: &str =
    include_str!("../../../../backend_tests/sqlite-store/fixtures/history/v1/001_jobs.sql");
const V2_JOBS: &str =
    include_str!("../../../../backend_tests/sqlite-store/fixtures/history/v2/001_jobs.sql");
const V2_OUTBOX: &str =
    include_str!("../../../../backend_tests/sqlite-store/fixtures/history/v2/001_job_outbox.sql");
const V2_UPGRADE: &[&str] = &[
    include_str!("../../../../backend_tests/sqlite-store/fixtures/history/v2/002_jobs_prepare.sql"),
    include_str!("../../../../backend_tests/sqlite-store/fixtures/history/v2/002_job_outbox.sql"),
    include_str!(
        "../../../../backend_tests/sqlite-store/fixtures/history/v2/002_jobs_finalize.sql"
    ),
];
const V3: &[&str] = &[
    include_str!("../migrations/003_jobs.sql"),
    include_str!("../migrations/003_job_outbox.sql"),
];

#[test]
fn actual_tenant_schema_history_is_rejected_without_rewriting_rows_or_files() {
    for version in [1, 2] {
        let path = temporary_database();
        let mut connection = Connection::open(&path).unwrap();
        connection
            .pragma_update(None, "foreign_keys", true)
            .unwrap();
        let transaction = connection.transaction().unwrap();
        if version == 1 {
            transaction.execute_batch(V1).unwrap();
        } else {
            transaction.execute_batch(V2_JOBS).unwrap();
            transaction.execute_batch(V2_OUTBOX).unwrap();
        }
        let tenant = Uuid::now_v7().to_string();
        let job = Uuid::now_v7().to_string();
        transaction
            .execute(
                "INSERT INTO jobs VALUES (?1, ?2, 'retry-key', 'fixture-input', 'pending')",
                params![tenant, job],
            )
            .unwrap();
        transaction.execute(
            "INSERT INTO job_outbox (tenant_id, event_id, job_id, kind) VALUES (?1, ?2, ?3, 'job_submitted')",
            params![tenant, Uuid::now_v7().to_string(), job],
        ).unwrap();
        if version == 2 {
            for sql in V2_UPGRADE {
                transaction.execute_batch(sql).unwrap();
            }
        }
        transaction
            .pragma_update(None, "user_version", version)
            .unwrap();
        transaction.commit().unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        assert!(
            matches!(SqliteStore::open(&path), Err(StoreError::UnsupportedSchema(v)) if v == version)
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        let connection = Connection::open(&path).unwrap();
        let row: (String, String) = connection
            .query_row(
                "SELECT tenant_id, input_ref FROM jobs WHERE job_id = ?1",
                [&job],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(row, (tenant, "fixture-input".into()));
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM job_outbox", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(connection);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn existing_owned_schema_is_opened_unchanged_and_keeps_idempotency() {
    let path = temporary_database();
    let connection = Connection::open(&path).unwrap();
    for sql in V3 {
        connection.execute_batch(sql).unwrap();
    }
    connection
        .pragma_update(None, "application_id", 0x4F52_4348_i64)
        .unwrap();
    connection.pragma_update(None, "user_version", 3).unwrap();
    drop(connection);
    let owner = JobOwnerId::parse(Uuid::now_v7()).unwrap();
    let mut original = SqliteStore::open(&path).unwrap();
    let SubmitOutcome::Created(job) = original
        .submit(owner, "retry-key", "fixture-input")
        .unwrap()
    else {
        panic!("new submission must be created");
    };
    drop(original);
    let before = fs::read(&path).unwrap();
    let mut reopened = SqliteStore::open(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(
        reopened
            .submit(owner, "retry-key", "fixture-input")
            .unwrap(),
        SubmitOutcome::Existing(job)
    );
    assert_eq!(reopened.pending_events(owner, 10).unwrap().len(), 1);
    drop(reopened);
    fs::remove_file(path).unwrap();
}

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-history-{}.sqlite", Uuid::now_v7()))
}
