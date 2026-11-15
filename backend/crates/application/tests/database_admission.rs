use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Waker},
    time::Duration,
};

use orchestrata_application::database_admission::{
    AdmissionError, AdmissionLimit, AdmissionScope, DatabaseAdmission, ScopeLimits,
};

const LONG_WAIT: Duration = Duration::from_secs(30);

fn scope(tenant: u128, principal: u128) -> AdmissionScope {
    AdmissionScope { tenant, principal }
}

fn limits(total: usize, tenant: usize, principal: usize) -> ScopeLimits {
    ScopeLimits::new(total, tenant, principal).expect("test limits must be valid")
}

fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
}

#[test]
fn fixed_limits_distinguish_principal_tenant_and_global_saturation() {
    assert_eq!(ScopeLimits::new(0, 0, 0), None);
    assert_eq!(ScopeLimits::new(1, 2, 1), None);
    assert_eq!(ScopeLimits::new(2, 1, 2), None);
    let control = DatabaseAdmission::new(limits(3, 2, 1), None);
    let first = control.try_acquire(scope(1, 10)).unwrap();
    assert_eq!(
        control.try_acquire(scope(1, 10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    let second = control.try_acquire(scope(1, 11)).unwrap();
    assert_eq!(
        control.try_acquire(scope(1, 12)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Tenant))
    );
    let third = control.try_acquire(scope(2, 10)).unwrap();
    assert_eq!(
        control.try_acquire(scope(3, 10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Total))
    );
    drop(first);
    assert!(control.try_acquire(scope(3, 10)).is_ok());
    drop((second, third));
    assert!(control.try_acquire(scope(1, 10)).is_ok());
}

#[tokio::test(start_paused = true)]
async fn waiting_is_bounded_by_scope_and_timeouts_and_cancellation_reclaim_slots() {
    let control = DatabaseAdmission::new(limits(1, 1, 1), Some(limits(3, 2, 1)));
    let active = control.try_acquire(scope(1, 10)).unwrap();
    let mut first = Box::pin(control.acquire_timeout(scope(1, 10), LONG_WAIT));
    assert_pending(first.as_mut());
    assert_eq!(
        control.acquire_timeout(scope(1, 10), LONG_WAIT).await.err(),
        Some(AdmissionError::WaitingFull(AdmissionLimit::Principal))
    );
    let mut second = Box::pin(control.acquire_timeout(scope(1, 11), LONG_WAIT));
    assert_pending(second.as_mut());
    assert_eq!(
        control.acquire_timeout(scope(1, 12), LONG_WAIT).await.err(),
        Some(AdmissionError::WaitingFull(AdmissionLimit::Tenant))
    );
    let mut third = Box::pin(control.acquire_timeout(scope(2, 12), LONG_WAIT));
    assert_pending(third.as_mut());
    assert_eq!(
        control.acquire_timeout(scope(3, 13), LONG_WAIT).await.err(),
        Some(AdmissionError::WaitingFull(AdmissionLimit::Total))
    );

    drop((first, second, third));
    for _ in 0..2 {
        assert_eq!(
            control
                .acquire_timeout(scope(1, 10), Duration::from_millis(1))
                .await
                .err(),
            Some(AdmissionError::TimedOut)
        );
    }
    drop(active);
    assert!(control.try_acquire(scope(1, 10)).is_ok());
}

#[tokio::test]
async fn dispatch_skips_waiter_at_its_own_limit_and_reserves_for_eligible_scope() {
    let control = DatabaseAdmission::new(limits(3, 2, 1), Some(limits(3, 2, 2)));
    let tenant1_user1 = control.try_acquire(scope(1, 10)).unwrap();
    let tenant1_user2 = control.try_acquire(scope(1, 11)).unwrap();
    let tenant2_user1 = control.try_acquire(scope(2, 10)).unwrap();
    let mut at_limit = Box::pin(control.acquire_timeout(scope(1, 10), LONG_WAIT));
    assert_pending(at_limit.as_mut());
    let mut unrelated = Box::pin(control.acquire_timeout(scope(3, 10), LONG_WAIT));
    assert_pending(unrelated.as_mut());

    drop(tenant2_user1);
    assert_pending(at_limit.as_mut());
    assert_eq!(
        control.try_acquire(scope(4, 10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Total))
    );
    let unrelated_permit = unrelated.await.unwrap();

    drop(tenant1_user1);
    let earlier_permit = at_limit.await.unwrap();
    drop((earlier_permit, unrelated_permit, tenant1_user2));
}

#[tokio::test]
async fn eligible_fifo_and_cancel_after_grant_transfer_capacity_without_an_extra_release() {
    let control = DatabaseAdmission::new(limits(1, 1, 1), Some(limits(2, 1, 1)));
    let active = control.try_acquire(scope(1, 10)).unwrap();
    let mut first = Box::pin(control.acquire_timeout(scope(2, 10), LONG_WAIT));
    assert_pending(first.as_mut());
    let mut second = Box::pin(control.acquire_timeout(scope(3, 10), LONG_WAIT));
    assert_pending(second.as_mut());

    drop(active);
    assert_pending(second.as_mut());
    assert_eq!(
        control.try_acquire(scope(4, 10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Total))
    );
    drop(first);

    let next = second.await.unwrap();
    assert_eq!(
        control.try_acquire(scope(4, 10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Total))
    );
    drop(next);
    assert!(control.try_acquire(scope(4, 10)).is_ok());
}

#[tokio::test]
async fn zero_timeout_and_disabled_queue_do_not_wait() {
    let queued = DatabaseAdmission::new(limits(1, 1, 1), Some(limits(1, 1, 1)));
    let unqueued = DatabaseAdmission::new(limits(1, 1, 1), None);
    let queued_active = queued.try_acquire(scope(1, 10)).unwrap();
    let unqueued_active = unqueued.try_acquire(scope(1, 10)).unwrap();
    assert_eq!(
        queued
            .acquire_timeout(scope(1, 10), Duration::ZERO)
            .await
            .err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    assert_eq!(
        unqueued
            .acquire_timeout(scope(1, 10), LONG_WAIT)
            .await
            .err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    let mut waiter = Box::pin(queued.acquire_timeout(scope(1, 10), LONG_WAIT));
    assert_pending(waiter.as_mut());
    drop(queued_active);
    drop(waiter.await.unwrap());
    drop(unqueued_active);
}

#[test]
fn concurrent_callers_share_the_same_total_ceiling() {
    let control = DatabaseAdmission::new(limits(6, 3, 2), None);
    let active = AtomicUsize::new(0);
    let barrier = std::sync::Barrier::new(24);

    std::thread::scope(|threads| {
        for index in 0..24 {
            let control = &control;
            let active = &active;
            let barrier = &barrier;
            threads.spawn(move || {
                barrier.wait();
                let permit = control.try_acquire(scope(index % 3, index % 2)).ok();
                let prior = permit
                    .as_ref()
                    .map(|_| active.fetch_add(1, Ordering::SeqCst));
                barrier.wait();
                if let Some(prior) = prior {
                    active.fetch_sub(1, Ordering::SeqCst);
                    assert!(prior < 6);
                }
                drop(permit);
            });
        }
    });

    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert!(control.try_acquire(scope(1, 1)).is_ok());
}
