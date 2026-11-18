use std::{fs, path::PathBuf};

use orchestrata_sqlite_store::{JobOwnerId, SqliteStore, StoreError};
use rusqlite::{Connection, ErrorCode, params};
use uuid::Uuid;

#[test]
fn incompatible_versions_preserve_existing_rows_and_schema_byte_for_byte() {
    for version in [1, 2] {
        let path = temporary_database();
        let older = Connection::open(&path).unwrap();
        older
            .execute_batch(
                "CREATE TABLE jobs (
                     partition_key TEXT NOT NULL,
                     job_id TEXT NOT NULL,
                     PRIMARY KEY (partition_key, job_id)
                 );
                 INSERT INTO jobs VALUES ('old-a', 'same-id'), ('old-b', 'same-id');",
            )
            .unwrap();
        older.pragma_update(None, "user_version", version).unwrap();
        drop(older);
        let before = fs::read(&path).unwrap();

        assert!(matches!(
            SqliteStore::open(&path),
            Err(StoreError::UnsupportedSchema(actual)) if actual == version
        ));
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn unmarked_or_unversioned_foreign_database_is_not_initialized() {
    for version in [0, 3] {
        let path = temporary_database();
        let other = Connection::open(&path).unwrap();
        other
            .execute_batch("CREATE TABLE unrelated (value INTEGER);")
            .unwrap();
        other.pragma_update(None, "user_version", version).unwrap();
        drop(other);
        let before = fs::read(&path).unwrap();

        assert!(matches!(
            SqliteStore::open(&path),
            Err(StoreError::IncompatibleDatabase)
        ));
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn job_identity_is_unique_across_owners_and_outbox_references_the_job() {
    let path = temporary_database();
    let store = SqliteStore::open(&path).unwrap();
    let connection = Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let first = Uuid::now_v7().to_string();
    let second = Uuid::now_v7().to_string();
    let job = Uuid::now_v7().to_string();
    let insert = "INSERT INTO jobs (job_id, owner_id, idempotency_key, input_ref, status)
                  VALUES (?1, ?2, 'scope_key', 'fixture', 'pending')";
    connection.execute(insert, params![job, first]).unwrap();
    let duplicate = connection
        .execute(insert, params![job, second])
        .unwrap_err();
    assert_eq!(
        duplicate.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );
    let missing = connection
        .execute(
            "INSERT INTO job_outbox (event_id, job_id, kind) VALUES (?1, ?2, 'job_submitted')",
            params![Uuid::now_v7().to_string(), Uuid::now_v7().to_string()],
        )
        .unwrap_err();
    assert_eq!(
        missing.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );
    drop((connection, store));
    fs::remove_file(path).unwrap();
}

#[test]
fn owner_identity_requires_valid_rfc_variant_and_v7() {
    assert_eq!(JobOwnerId::parse(Uuid::nil()), None);
    let good = Uuid::now_v7();
    let mut bytes = *good.as_bytes();
    bytes[8] &= 0b0111_1111;
    assert_eq!(JobOwnerId::parse(Uuid::from_bytes(bytes)), None);
    assert_eq!(JobOwnerId::parse(good).unwrap().as_uuid(), good);
}

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-schema-{}.sqlite", Uuid::now_v7()))
}
