use std::{
    future::Future,
    pin::Pin,
    sync::{
        Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Waker},
    time::Duration,
};

use orchestrata_application::database_admission::{
    AdmissionError, AdmissionLimit, AdmissionLimits, AdmissionPrincipal, DatabaseAdmission,
};

const LONG_WAIT: Duration = Duration::from_secs(30);

fn principal(id: u128) -> AdmissionPrincipal {
    AdmissionPrincipal::from_trusted_id(id)
}

fn limits(global: usize, per_principal: usize) -> AdmissionLimits {
    AdmissionLimits::new(global, per_principal).expect("test limits must be valid")
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
fn active_limits_distinguish_principal_and_global_saturation() {
    assert_eq!(AdmissionLimits::new(0, 0), None);
    assert_eq!(AdmissionLimits::new(1, 0), None);
    assert_eq!(AdmissionLimits::new(1, 2), None);

    let control = DatabaseAdmission::new(limits(2, 1), None);
    let first = control.try_acquire(principal(10)).unwrap();
    assert_eq!(
        control.try_acquire(principal(10)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    let second = control.try_acquire(principal(11)).unwrap();
    assert_eq!(
        control.try_acquire(principal(12)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Global))
    );

    drop(first);
    assert!(control.try_acquire(principal(12)).is_ok());
    drop(second);
    assert!(control.try_acquire(principal(10)).is_ok());
}

#[tokio::test(start_paused = true)]
async fn bounded_waiters_release_capacity_after_cancellation_and_timeout() {
    let control = DatabaseAdmission::new(limits(1, 1), Some(limits(2, 1)));
    let active = control.try_acquire(principal(10)).unwrap();
    let mut first = Box::pin(control.acquire_timeout(principal(10), LONG_WAIT));
    assert_pending(first.as_mut());
    assert_eq!(
        control
            .acquire_timeout(principal(10), LONG_WAIT)
            .await
            .err(),
        Some(AdmissionError::WaitingFull(AdmissionLimit::Principal))
    );

    let mut second = Box::pin(control.acquire_timeout(principal(11), LONG_WAIT));
    assert_pending(second.as_mut());
    assert_eq!(
        control
            .acquire_timeout(principal(12), LONG_WAIT)
            .await
            .err(),
        Some(AdmissionError::WaitingFull(AdmissionLimit::Global))
    );

    drop((first, second));
    for _ in 0..2 {
        assert_eq!(
            control
                .acquire_timeout(principal(10), Duration::from_millis(1))
                .await
                .err(),
            Some(AdmissionError::TimedOut)
        );
    }
    drop(active);
    assert!(control.try_acquire(principal(10)).is_ok());
}

#[tokio::test]
async fn dispatch_skips_waiters_that_are_still_at_their_principal_limit() {
    let control = DatabaseAdmission::new(limits(3, 1), Some(limits(3, 2)));
    let first = control.try_acquire(principal(10)).unwrap();
    let second = control.try_acquire(principal(11)).unwrap();
    let third = control.try_acquire(principal(12)).unwrap();
    let mut at_limit = Box::pin(control.acquire_timeout(principal(10), LONG_WAIT));
    assert_pending(at_limit.as_mut());
    let mut unrelated = Box::pin(control.acquire_timeout(principal(13), LONG_WAIT));
    assert_pending(unrelated.as_mut());

    drop(third);
    assert_pending(at_limit.as_mut());
    assert_eq!(
        control.try_acquire(principal(14)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Global))
    );
    let unrelated_permit = unrelated.await.unwrap();

    drop(first);
    let earlier_permit = at_limit.await.unwrap();
    drop((earlier_permit, unrelated_permit, second));
}

#[tokio::test]
async fn eligible_fifo_and_cancel_after_grant_transfer_capacity_without_an_extra_release() {
    let control = DatabaseAdmission::new(limits(1, 1), Some(limits(2, 1)));
    let active = control.try_acquire(principal(10)).unwrap();
    let mut first = Box::pin(control.acquire_timeout(principal(11), LONG_WAIT));
    assert_pending(first.as_mut());
    let mut second = Box::pin(control.acquire_timeout(principal(12), LONG_WAIT));
    assert_pending(second.as_mut());

    drop(active);
    assert_pending(second.as_mut());
    assert_eq!(
        control.try_acquire(principal(13)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Global))
    );
    drop(first);

    let next = second.await.unwrap();
    assert_eq!(
        control.try_acquire(principal(13)).err(),
        Some(AdmissionError::Busy(AdmissionLimit::Global))
    );
    drop(next);
    assert!(control.try_acquire(principal(13)).is_ok());
}

#[tokio::test]
async fn zero_timeout_and_disabled_queue_do_not_wait() {
    let queued = DatabaseAdmission::new(limits(1, 1), Some(limits(1, 1)));
    let unqueued = DatabaseAdmission::new(limits(1, 1), None);
    let queued_active = queued.try_acquire(principal(10)).unwrap();
    let unqueued_active = unqueued.try_acquire(principal(10)).unwrap();
    assert_eq!(
        queued
            .acquire_timeout(principal(10), Duration::ZERO)
            .await
            .err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    assert_eq!(
        unqueued
            .acquire_timeout(principal(10), LONG_WAIT)
            .await
            .err(),
        Some(AdmissionError::Busy(AdmissionLimit::Principal))
    );
    let mut waiter = Box::pin(queued.acquire_timeout(principal(10), LONG_WAIT));
    assert_pending(waiter.as_mut());
    drop(queued_active);
    drop(waiter.await.unwrap());
    drop(unqueued_active);
}

#[test]
fn concurrent_callers_obey_the_shared_global_and_principal_ceilings() {
    const CALLERS: usize = 24;
    const PRINCIPALS: usize = 4;
    const GLOBAL_LIMIT: usize = 6;
    const PRINCIPAL_LIMIT: usize = 2;

    let control = DatabaseAdmission::new(limits(GLOBAL_LIMIT, PRINCIPAL_LIMIT), None);
    let active = AtomicUsize::new(0);
    let principal_usage: [AtomicUsize; PRINCIPALS] = std::array::from_fn(|_| AtomicUsize::new(0));
    let barrier = Barrier::new(CALLERS + 1);

    std::thread::scope(|threads| {
        for index in 0..CALLERS {
            let control = &control;
            let active = &active;
            let principal_usage = &principal_usage[index % PRINCIPALS];
            let barrier = &barrier;
            threads.spawn(move || {
                barrier.wait();

                let permit = control
                    .try_acquire(principal((index % PRINCIPALS) as u128))
                    .ok();
                if permit.is_some() {
                    active.fetch_add(1, Ordering::SeqCst);
                    principal_usage.fetch_add(1, Ordering::SeqCst);
                }
                barrier.wait();
                barrier.wait();

                if permit.is_some() {
                    active.fetch_sub(1, Ordering::SeqCst);
                    principal_usage.fetch_sub(1, Ordering::SeqCst);
                }
                drop(permit);
            });
        }

        barrier.wait();
        barrier.wait();
        let observed_global = active.load(Ordering::SeqCst);
        let within_principal_limits = principal_usage
            .iter()
            .all(|count| count.load(Ordering::SeqCst) <= PRINCIPAL_LIMIT);
        barrier.wait();

        assert_eq!(observed_global, GLOBAL_LIMIT);
        assert!(within_principal_limits);
    });

    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert!(control.try_acquire(principal(1)).is_ok());
}
