use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

use orchestrata_core::ids::job_id::JobId;
use orchestrata_sqlite_store::{
    CancelOutcome, CompleteOutcome, JobEventKind, JobOwnerId, SqliteStore, StoreError,
    SubmitOutcome,
};
use rusqlite::{Connection, params};
use uuid::Uuid;

const LEASE_DURATION: Duration = Duration::from_secs(60);
#[test]
fn restarted_worker_reclaims_expired_job_and_only_current_lease_can_complete() {
    let owner = owner_id();
    let outsider = owner_id();
    let path = temporary_database();
    let mut original = SqliteStore::open(&path).unwrap();
    let job = submit(&mut original, owner, "first");
    let previous = original.claim_next(LEASE_DURATION).unwrap().unwrap();
    assert_eq!(previous.job_id(), job);
    assert_eq!(previous.owner_id(), owner);
    assert_eq!(previous.input_ref(), "input/first");
    assert!(previous.expires_at_unix_ms() > 0);
    drop(original);

    let mut restarted = SqliteStore::open(&path).unwrap();
    assert!(restarted.claim_next(LEASE_DURATION).unwrap().is_none());
    assert_eq!(
        restarted.cancel_pending(owner, job).unwrap(),
        CancelOutcome::NotPending
    );
    expire(&path, owner, job);
    assert_eq!(
        restarted.complete(&previous, "result/expired").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );

    let lease = restarted.claim_next(LEASE_DURATION).unwrap().unwrap();
    assert_eq!(lease.job_id(), job);
    assert_eq!(
        restarted.complete(&previous, "result/stale").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );
    assert_eq!(restarted.completed_result(owner, job).unwrap(), None);
    assert_eq!(restarted.pending_events(owner, 10).unwrap().len(), 1);
    assert_eq!(
        restarted.complete(&lease, "result/current").unwrap(),
        CompleteOutcome::Completed,
    );
    assert_eq!(
        restarted.complete(&lease, "result/duplicate").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );
    assert_eq!(
        restarted.cancel_pending(owner, job).unwrap(),
        CancelOutcome::NotPending
    );
    assert!(restarted.claim_next(LEASE_DURATION).unwrap().is_none());
    drop(restarted);

    let reader = SqliteStore::open(&path).unwrap();
    assert_eq!(
        reader.completed_result(owner, job).unwrap().as_deref(),
        Some("result/current")
    );
    assert_eq!(reader.completed_result(outsider, job).unwrap(), None);
    let events = reader.pending_events(owner, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].kind, JobEventKind::Completed);
    assert_eq!(events[1].job_id, job);
    drop(reader);
    fs::remove_file(path).unwrap();
}

#[test]
fn completion_and_result_reference_roll_back_if_event_insert_fails() {
    let path = temporary_database();
    let owner = owner_id();
    let mut store = SqliteStore::open(&path).unwrap();
    let job = submit(&mut store, owner, "first");
    let lease = store.claim_next(LEASE_DURATION).unwrap().unwrap();
    let observer = Connection::open(&path).unwrap();
    observer
        .execute_batch(
            "CREATE TRIGGER reject_completion BEFORE INSERT ON job_outbox
             WHEN NEW.kind = 'job_completed'
             BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
        )
        .unwrap();

    assert!(matches!(
        store.complete(&lease, "result/rollback"),
        Err(StoreError::Database(_))
    ));
    assert_eq!(store.completed_result(owner, job).unwrap(), None);
    assert!(store.claim_next(LEASE_DURATION).unwrap().is_none());
    assert_eq!(store.pending_events(owner, 10).unwrap().len(), 1);
    observer
        .execute_batch("DROP TRIGGER reject_completion;")
        .unwrap();
    assert_eq!(
        store.complete(&lease, "result/committed").unwrap(),
        CompleteOutcome::Completed
    );
    assert_eq!(
        store.completed_result(owner, job).unwrap().as_deref(),
        Some("result/committed")
    );
    assert_eq!(store.pending_events(owner, 10).unwrap().len(), 2);
    drop((observer, store));
    fs::remove_file(path).unwrap();
}

#[test]
fn two_connections_cannot_claim_the_same_job() {
    let path = temporary_database();
    let owner = owner_id();
    let mut store = SqliteStore::open(&path).unwrap();
    let job = submit(&mut store, owner, "first");
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut connection = SqliteStore::open(path).unwrap();
                barrier.wait();
                connection
                    .claim_next(LEASE_DURATION)
                    .unwrap()
                    .map(|lease| lease.job_id())
            })
        })
        .collect();
    let claims: Vec<_> = threads
        .into_iter()
        .filter_map(|t| t.join().unwrap())
        .collect();
    assert_eq!(claims, vec![job]);
    assert_eq!(store.pending_events(owner, 10).unwrap().len(), 1);
    drop(store);
    fs::remove_file(path).unwrap();
}

#[test]
fn cancelled_jobs_stay_cancelled_and_invalid_references_do_not_consume_the_lease() {
    let owner = owner_id();
    let mut store = SqliteStore::open(":memory:").unwrap();
    let cancelled = submit(&mut store, owner, "cancelled");
    assert_eq!(
        store.cancel_pending(owner, cancelled).unwrap(),
        CancelOutcome::Cancelled
    );
    assert!(store.claim_next(LEASE_DURATION).unwrap().is_none());
    for invalid in [
        Duration::ZERO,
        Duration::from_micros(999),
        Duration::from_secs(86_401),
    ] {
        assert!(matches!(
            store.claim_next(invalid),
            Err(StoreError::InvalidLeaseDuration)
        ));
    }

    let job = submit(&mut store, owner, "second");
    let lease = store.claim_next(LEASE_DURATION).unwrap().unwrap();
    assert_eq!(lease.job_id(), job);
    assert!(matches!(
        store.complete(&lease, "\n"),
        Err(StoreError::InvalidResultReference)
    ));
    assert_eq!(
        store.complete(&lease, "result/valid").unwrap(),
        CompleteOutcome::Completed
    );
}

#[test]
fn trusted_worker_claims_across_owners_but_results_are_owner_scoped() {
    let mut store = SqliteStore::open(":memory:").unwrap();
    let one = owner_id();
    let two = owner_id();
    let first = submit(&mut store, one, "same_key");
    let second = submit(&mut store, two, "same_key");
    assert_ne!(first, second);

    let mut claimed = Vec::new();
    while let Some(lease) = store.claim_next(LEASE_DURATION).unwrap() {
        let (owner, other) = if lease.job_id() == first {
            (one, two)
        } else {
            (two, one)
        };
        assert_eq!(lease.owner_id(), owner);
        assert_eq!(
            store.complete(&lease, "result/committed").unwrap(),
            CompleteOutcome::Completed
        );
        assert_eq!(
            store
                .completed_result(owner, lease.job_id())
                .unwrap()
                .as_deref(),
            Some("result/committed")
        );
        assert_eq!(store.completed_result(other, lease.job_id()).unwrap(), None);
        claimed.push(lease.job_id());
    }
    assert_eq!(claimed.len(), 2);
    assert!(claimed.contains(&first) && claimed.contains(&second));
}

fn submit(store: &mut SqliteStore, owner: JobOwnerId, key: &str) -> JobId {
    let SubmitOutcome::Created(job) = store.submit(owner, key, &format!("input/{key}")).unwrap()
    else {
        panic!("expected a newly submitted job")
    };
    job
}

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-lease-{}.sqlite", Uuid::now_v7()))
}

fn expire(path: &Path, owner: JobOwnerId, job: JobId) {
    let connection = Connection::open(path).unwrap();
    assert_eq!(
        connection
            .execute(
                "UPDATE jobs SET lease_expires_ms = 1 WHERE owner_id = ?1 AND job_id = ?2",
                params![owner.as_uuid().to_string(), job.as_uuid().to_string()],
            )
            .unwrap(),
        1,
    );
}

fn owner_id() -> JobOwnerId {
    JobOwnerId::parse(Uuid::now_v7()).unwrap()
}
