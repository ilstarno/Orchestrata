use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

use orchestrata_core::ids::{job_id::JobId, tenant_id::TenantId};
use orchestrata_sqlite_store::{
    CancelOutcome, CompleteOutcome, JobEventKind, SqliteStore, StoreError, SubmitOutcome,
};
use rusqlite::{Connection, params};
use uuid::Uuid;

const LEASE_DURATION: Duration = Duration::from_secs(60);
const LEGACY_JOBS_SCHEMA: &str = r#"CREATE TABLE jobs (
    tenant_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    input_ref TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'cancelled')),
    PRIMARY KEY (tenant_id, job_id),
    UNIQUE (tenant_id, idempotency_key)
) STRICT;"#;
const LEGACY_OUTBOX_SCHEMA: &str = r#"CREATE TABLE job_outbox (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    event_id TEXT NOT NULL UNIQUE,
    job_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('job_submitted', 'job_cancelled')),
    published INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1)),
    FOREIGN KEY (tenant_id, job_id) REFERENCES jobs (tenant_id, job_id)
) STRICT;

CREATE INDEX job_outbox_pending ON job_outbox (tenant_id, sequence) WHERE published = 0;"#;

#[test]
fn restarted_worker_reclaims_expired_job_and_only_current_lease_can_complete() {
    let tenant = TenantId::new();
    let outsider = TenantId::new();
    let path = temporary_database();
    let mut original = SqliteStore::open(&path).unwrap();
    let job = submit(&mut original, tenant, "first");
    let previous = original
        .claim_next(tenant, LEASE_DURATION)
        .unwrap()
        .unwrap();
    assert_eq!(previous.job_id(), job);
    assert_eq!(previous.input_ref(), "input/first");
    assert!(previous.expires_at_unix_ms() > 0);
    drop(original);

    let mut restarted = SqliteStore::open(&path).unwrap();
    assert!(
        restarted
            .claim_next(outsider, LEASE_DURATION)
            .unwrap()
            .is_none()
    );
    assert!(
        restarted
            .claim_next(tenant, LEASE_DURATION)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        restarted.cancel_pending(tenant, job).unwrap(),
        CancelOutcome::NotPending
    );
    expire(&path, tenant, job);
    assert_eq!(
        restarted.complete(&previous, "result/expired").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );

    let lease = restarted
        .claim_next(tenant, LEASE_DURATION)
        .unwrap()
        .unwrap();
    assert_eq!(lease.job_id(), job);
    assert_eq!(
        restarted.complete(&previous, "result/stale").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );
    assert_eq!(restarted.completed_result(tenant, job).unwrap(), None);
    assert_eq!(restarted.pending_events(tenant, 10).unwrap().len(), 1);
    assert_eq!(
        restarted.complete(&lease, "result/current").unwrap(),
        CompleteOutcome::Completed,
    );
    assert_eq!(
        restarted.complete(&lease, "result/duplicate").unwrap(),
        CompleteOutcome::LeaseNotCurrent,
    );
    assert_eq!(
        restarted.cancel_pending(tenant, job).unwrap(),
        CancelOutcome::NotPending
    );
    assert!(
        restarted
            .claim_next(tenant, LEASE_DURATION)
            .unwrap()
            .is_none()
    );
    drop(restarted);

    let reader = SqliteStore::open(&path).unwrap();
    assert_eq!(
        reader.completed_result(tenant, job).unwrap().as_deref(),
        Some("result/current")
    );
    assert_eq!(reader.completed_result(outsider, job).unwrap(), None);
    let events = reader.pending_events(tenant, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].kind, JobEventKind::Completed);
    assert_eq!(events[1].job_id, job);
    drop(reader);
    fs::remove_file(path).unwrap();
}

#[test]
fn completion_and_result_reference_roll_back_if_event_insert_fails() {
    let path = temporary_database();
    let tenant = TenantId::new();
    let mut store = SqliteStore::open(&path).unwrap();
    let job = submit(&mut store, tenant, "first");
    let lease = store.claim_next(tenant, LEASE_DURATION).unwrap().unwrap();
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
    assert_eq!(store.completed_result(tenant, job).unwrap(), None);
    assert!(store.claim_next(tenant, LEASE_DURATION).unwrap().is_none());
    assert_eq!(store.pending_events(tenant, 10).unwrap().len(), 1);
    observer
        .execute_batch("DROP TRIGGER reject_completion;")
        .unwrap();
    assert_eq!(
        store.complete(&lease, "result/committed").unwrap(),
        CompleteOutcome::Completed
    );
    assert_eq!(
        store.completed_result(tenant, job).unwrap().as_deref(),
        Some("result/committed")
    );
    assert_eq!(store.pending_events(tenant, 10).unwrap().len(), 2);
    drop((observer, store));
    fs::remove_file(path).unwrap();
}

#[test]
fn two_connections_cannot_claim_the_same_job() {
    let path = temporary_database();
    let tenant = TenantId::new();
    let mut store = SqliteStore::open(&path).unwrap();
    let job = submit(&mut store, tenant, "first");
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut connection = SqliteStore::open(path).unwrap();
                barrier.wait();
                connection
                    .claim_next(tenant, LEASE_DURATION)
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
    assert_eq!(store.pending_events(tenant, 10).unwrap().len(), 1);
    drop(store);
    fs::remove_file(path).unwrap();
}

#[test]
fn cancelled_jobs_stay_cancelled_and_invalid_references_do_not_consume_the_lease() {
    let tenant = TenantId::new();
    let mut store = SqliteStore::open(":memory:").unwrap();
    let cancelled = submit(&mut store, tenant, "cancelled");
    assert_eq!(
        store.cancel_pending(tenant, cancelled).unwrap(),
        CancelOutcome::Cancelled
    );
    assert!(store.claim_next(tenant, LEASE_DURATION).unwrap().is_none());
    for invalid in [
        Duration::ZERO,
        Duration::from_micros(999),
        Duration::from_secs(86_401),
    ] {
        assert!(matches!(
            store.claim_next(tenant, invalid),
            Err(StoreError::InvalidLeaseDuration)
        ));
    }

    let job = submit(&mut store, tenant, "second");
    let lease = store.claim_next(tenant, LEASE_DURATION).unwrap().unwrap();
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
fn migration_preserves_jobs_event_identity_acknowledgements_and_order() {
    let path = temporary_database();
    let tenant = TenantId::new();
    let pending_job = JobId::new();
    let cancelled_job = JobId::new();
    let published_event = Uuid::now_v7();
    let queued_event = Uuid::now_v7();
    let tenant_key = tenant.as_uuid().to_string();
    let legacy = Connection::open(&path).unwrap();
    legacy
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA user_version = 1;")
        .unwrap();
    legacy.execute_batch(LEGACY_JOBS_SCHEMA).unwrap();
    legacy.execute_batch(LEGACY_OUTBOX_SCHEMA).unwrap();
    for (job, key, status) in [
        (pending_job, "pending", "pending"),
        (cancelled_job, "cancelled", "cancelled"),
    ] {
        legacy
            .execute(
                "INSERT INTO jobs (tenant_id, job_id, idempotency_key, input_ref, status)
             VALUES (?1, ?2, ?3, 'fixture', ?4)",
                params![tenant_key, job.as_uuid().to_string(), key, status],
            )
            .unwrap();
    }
    for (job, event, kind, published) in [
        (pending_job, published_event, "job_submitted", 1),
        (cancelled_job, queued_event, "job_cancelled", 0),
    ] {
        legacy
            .execute(
                "INSERT INTO job_outbox (tenant_id, job_id, event_id, kind, published)
             VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    tenant_key,
                    job.as_uuid().to_string(),
                    event.to_string(),
                    kind,
                    published
                ],
            )
            .unwrap();
    }
    drop(legacy);

    let mut store = SqliteStore::open(&path).unwrap();
    assert_eq!(
        store.submit(tenant, "pending", "fixture").unwrap(),
        SubmitOutcome::Existing(pending_job)
    );
    assert_eq!(
        store.cancel_pending(tenant, cancelled_job).unwrap(),
        CancelOutcome::AlreadyCancelled
    );
    assert!(!store.mark_published(tenant, published_event).unwrap());
    let lease = store.claim_next(tenant, LEASE_DURATION).unwrap().unwrap();
    assert_eq!(lease.job_id(), pending_job);
    assert_eq!(
        store.complete(&lease, "result/after-migration").unwrap(),
        CompleteOutcome::Completed
    );
    let events = store.pending_events(tenant, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_id, queued_event);
    assert_eq!(events[0].kind, JobEventKind::Cancelled);
    assert_eq!(events[1].kind, JobEventKind::Completed);
    assert_eq!(events[1].job_id, pending_job);
    drop(store);
    fs::remove_file(path).unwrap();
}

fn submit(store: &mut SqliteStore, tenant: TenantId, key: &str) -> JobId {
    let SubmitOutcome::Created(job) = store.submit(tenant, key, &format!("input/{key}")).unwrap()
    else {
        panic!("expected a newly submitted job")
    };
    job
}

fn temporary_database() -> PathBuf {
    std::env::temp_dir().join(format!("orchestrata-lease-{}.sqlite", Uuid::now_v7()))
}

fn expire(path: &Path, tenant: TenantId, job: JobId) {
    let connection = Connection::open(path).unwrap();
    assert_eq!(
        connection
            .execute(
                "UPDATE jobs SET lease_expires_ms = 1 WHERE tenant_id = ?1 AND job_id = ?2",
                params![tenant.as_uuid().to_string(), job.as_uuid().to_string()],
            )
            .unwrap(),
        1,
    );
}
