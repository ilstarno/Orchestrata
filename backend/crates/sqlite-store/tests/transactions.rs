use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
};

use orchestrata_core::ids::job_id::JobId;
use orchestrata_sqlite_store::{
    CancelOutcome, JobEventKind, JobOwnerId, SqliteStore, StoreError, SubmitOutcome,
};
use rusqlite::Connection;
use uuid::Uuid;

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-test-{}.sqlite", Uuid::now_v7()))
}

fn created(outcome: SubmitOutcome) -> JobId {
    match outcome {
        SubmitOutcome::Created(job) => job,
        _ => panic!("expected a new job"),
    }
}

#[test]
fn retries_are_idempotent_only_for_the_same_owner_and_request() {
    let mut store = SqliteStore::open(":memory:").unwrap();
    let one = owner_id();
    let two = owner_id();
    let job = created(store.submit(one, "request", "fixture/one").unwrap());

    assert_eq!(
        store.submit(one, "request", "fixture/one").unwrap(),
        SubmitOutcome::Existing(job)
    );
    assert_eq!(
        store.submit(one, "request", "fixture/two").unwrap(),
        SubmitOutcome::ConflictingInput
    );
    let other = created(store.submit(two, "request", "fixture/two").unwrap());
    assert_eq!(store.pending_events(one, 10).unwrap()[0].job_id, job);
    assert_eq!(store.pending_events(two, 10).unwrap()[0].job_id, other);
    assert_eq!(store.pending_events(one, 10).unwrap().len(), 1);
    assert!(matches!(
        store.submit(one, "", "fixture/one"),
        Err(StoreError::InvalidSubmission)
    ));
}

#[test]
fn rolled_back_outbox_insert_also_rolls_back_submission() {
    let path = temporary_database();
    let mut store = SqliteStore::open(&path).unwrap();
    let other = Connection::open(&path).unwrap();
    other
        .execute_batch(
            "CREATE TRIGGER fail_event BEFORE INSERT ON job_outbox
         BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
        )
        .unwrap();

    let owner = owner_id();
    assert!(store.submit(owner, "request", "fixture/one").is_err());
    assert!(store.pending_events(owner, 10).unwrap().is_empty());
    other.execute_batch("DROP TRIGGER fail_event;").unwrap();

    assert!(matches!(
        store.submit(owner, "request", "fixture/one").unwrap(),
        SubmitOutcome::Created(_)
    ));
    assert_eq!(store.pending_events(owner, 10).unwrap().len(), 1);
    drop((store, other));
    fs::remove_file(path).unwrap();
}

#[test]
fn unacknowledged_events_survive_restart_with_the_same_identity() {
    let path = temporary_database();
    let owner = owner_id();
    let outsider = owner_id();
    let mut store = SqliteStore::open(&path).unwrap();
    let job = created(store.submit(owner, "request", "fixture").unwrap());
    let initial_event = store.pending_events(owner, 10).unwrap()[0];
    assert_eq!(
        store.cancel_pending(outsider, job).unwrap(),
        CancelOutcome::NotFound
    );
    assert_eq!(
        store.cancel_pending(owner, job).unwrap(),
        CancelOutcome::Cancelled
    );
    assert_eq!(
        store.cancel_pending(owner, job).unwrap(),
        CancelOutcome::AlreadyCancelled
    );
    drop(store);

    let mut store = SqliteStore::open(&path).unwrap();
    let events = store.pending_events(owner, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], initial_event);
    assert_eq!(events[1].kind, JobEventKind::Cancelled);
    assert_eq!(events[1].job_id, job);
    assert!(
        !store
            .mark_published(outsider, initial_event.event_id)
            .unwrap()
    );
    assert!(store.mark_published(owner, initial_event.event_id).unwrap());
    assert!(!store.mark_published(owner, initial_event.event_id).unwrap());
    assert_eq!(store.pending_events(owner, 10).unwrap(), vec![events[1]]);
    drop(store);
    fs::remove_file(path).unwrap();
}

#[test]
fn two_connections_commit_one_job_and_one_event() {
    let path = temporary_database();
    let result_store = SqliteStore::open(&path).unwrap();
    let owner = owner_id();
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut store = SqliteStore::open(path).unwrap();
                barrier.wait();
                store.submit(owner, "request", "fixture").unwrap()
            })
        })
        .collect();
    let states: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    let job = match states[..] {
        [SubmitOutcome::Created(a), SubmitOutcome::Existing(b)]
        | [SubmitOutcome::Existing(a), SubmitOutcome::Created(b)]
            if a == b =>
        {
            a
        }
        _ => panic!("expected one creation and one matching retry, got {states:?}"),
    };
    let events = result_store.pending_events(owner, 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].job_id, job);
    drop(result_store);
    fs::remove_file(path).unwrap();
}

#[test]
fn unsupported_schema_version_fails_closed() {
    let path = temporary_database();
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 4).unwrap();
    drop(connection);
    assert!(matches!(
        SqliteStore::open(&path),
        Err(StoreError::UnsupportedSchema(4))
    ));
    fs::remove_file(path).unwrap();
}

fn owner_id() -> JobOwnerId {
    JobOwnerId::parse(Uuid::now_v7()).unwrap()
}
