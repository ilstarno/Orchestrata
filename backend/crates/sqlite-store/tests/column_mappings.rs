use std::{fs, path::PathBuf};

use orchestrata_imports::{ColumnMapping, Mapping};
use orchestrata_sqlite_store::{
    ColumnMappingId, JobEventKind, JobOwnerId, SqliteStore, StoreError,
};
use rusqlite::{Connection, ErrorCode, params};
use uuid::{Uuid, Version};

#[test]
fn immutable_mapping_survives_reopen_and_is_visible_only_to_its_owner() {
    let path = temporary_database();
    let owner = owner_id();
    let outsider = owner_id();
    let mapping = example();
    let mut store = SqliteStore::open(&path).unwrap();
    let id = store.save_column_mapping(owner, &mapping).unwrap();
    assert_eq!(id.as_uuid().get_version(), Some(Version::SortRand));
    assert_eq!(ColumnMappingId::parse(id.as_uuid()), Some(id));
    assert_eq!(ColumnMappingId::parse(Uuid::nil()), None);
    drop(store);

    let reader = SqliteStore::open(&path).unwrap();
    assert_eq!(reader.column_mapping(owner, id).unwrap(), Some(mapping));
    assert_eq!(reader.column_mapping(outsider, id).unwrap(), None);
    let absent = ColumnMappingId::parse(Uuid::now_v7()).unwrap();
    assert_eq!(reader.column_mapping(owner, absent).unwrap(), None);
    drop(reader);
    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_invalid_or_future_mapping_versions_on_save_and_load() {
    let path = temporary_database();
    let owner = owner_id();
    let mut store = SqliteStore::open(&path).unwrap();
    let future_json = r#"{"version":2,"columns":[{"source":"Job","target":"job-name"}]}"#;
    let future: Mapping = serde_json::from_str(future_json).unwrap();
    assert!(matches!(
        store.save_column_mapping(owner, &future),
        Err(StoreError::UnsupportedMappingVersion(2))
    ));
    let invalid: Mapping = serde_json::from_str(r#"{"version":1,"columns":[]}"#).unwrap();
    assert!(matches!(
        store.save_column_mapping(owner, &invalid),
        Err(StoreError::InvalidMapping)
    ));

    let id = store.save_column_mapping(owner, &example()).unwrap();
    let connection = Connection::open(&path).unwrap();
    overwrite(&connection, id, future_json);
    assert!(matches!(
        store.column_mapping(owner, id),
        Err(StoreError::UnsupportedMappingVersion(2))
    ));
    assert_eq!(store.column_mapping(owner_id(), id).unwrap(), None);
    overwrite(&connection, id, "invalid JSON");
    assert!(matches!(
        store.column_mapping(owner, id),
        Err(StoreError::InvalidStoredMapping)
    ));
    drop((connection, store));
    fs::remove_file(path).unwrap();
}

#[test]
fn database_and_reader_reject_oversized_snapshots() {
    let path = temporary_database();
    let owner = owner_id();
    let mut store = SqliteStore::open(&path).unwrap();
    let id = store.save_column_mapping(owner, &example()).unwrap();
    let connection = Connection::open(&path).unwrap();
    let oversized = "x".repeat(65_537);
    let error = connection
        .execute(
            "UPDATE column_mappings SET mapping_json = ?1 WHERE mapping_id = ?2",
            params![oversized, id.as_uuid().to_string()],
        )
        .unwrap_err();
    assert_eq!(
        error.sqlite_error_code(),
        Some(ErrorCode::ConstraintViolation)
    );

    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    overwrite(&connection, id, &oversized);
    assert!(matches!(
        store.column_mapping(owner, id),
        Err(StoreError::InvalidStoredMapping)
    ));
    drop((connection, store));
    fs::remove_file(path).unwrap();
}

#[test]
fn upgrades_marked_version_three_without_rewriting_jobs_or_events() {
    let path = temporary_database();
    let owner = owner_id();
    let job = Uuid::now_v7();
    let event = Uuid::now_v7();
    let old = Connection::open(&path).unwrap();
    old.execute_batch(include_str!("../migrations/003_jobs.sql"))
        .unwrap();
    old.execute_batch(include_str!("../migrations/003_job_outbox.sql"))
        .unwrap();
    old.pragma_update(None, "application_id", 0x4F52_4348_u32)
        .unwrap();
    old.pragma_update(None, "user_version", 3).unwrap();
    old.execute(
        "INSERT INTO jobs (job_id, owner_id, idempotency_key, input_ref, status)
         VALUES (?1, ?2, 'saved_job', 'fixture', 'pending')",
        params![job.to_string(), owner.as_uuid().to_string()],
    )
    .unwrap();
    old.execute(
        "INSERT INTO job_outbox (event_id, job_id, kind) VALUES (?1, ?2, 'job_submitted')",
        params![event.to_string(), job.to_string()],
    )
    .unwrap();
    drop(old);

    let mut store = SqliteStore::open(&path).unwrap();
    let events = store.pending_events(owner, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_id, event);
    assert_eq!(events[0].job_id.as_uuid(), job);
    assert_eq!(events[0].kind, JobEventKind::Submitted);
    let mapping = example();
    let id = store.save_column_mapping(owner, &mapping).unwrap();
    assert_eq!(store.column_mapping(owner, id).unwrap(), Some(mapping));
    drop(store);

    let migrated = Connection::open(&path).unwrap();
    assert_eq!(
        migrated
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
    drop(migrated);
    fs::remove_file(path).unwrap();
}

fn example() -> Mapping {
    Mapping::new(vec![
        ColumnMapping::new("Job", "job-name"),
        ColumnMapping::new("Worker", "worker-name"),
    ])
    .unwrap()
}

fn overwrite(connection: &Connection, id: ColumnMappingId, text: &str) {
    assert_eq!(
        connection
            .execute(
                "UPDATE column_mappings SET mapping_json = ?1 WHERE mapping_id = ?2",
                params![text, id.as_uuid().to_string()],
            )
            .unwrap(),
        1
    );
}

fn owner_id() -> JobOwnerId {
    JobOwnerId::parse(Uuid::now_v7()).unwrap()
}

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-mapping-{}.sqlite", Uuid::now_v7()))
}
