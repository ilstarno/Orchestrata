use std::sync::mpsc;

use tokio::sync::oneshot;

use super::*;

#[tokio::test]
async fn rejects_saturation_without_running_submitted_work() {
    let admission = SolveAdmission::new(1);
    let worker = admission.clone();
    let (started, running) = oneshot::channel();
    let (release, blocked) = mpsc::channel();
    let task = tokio::spawn(async move {
        worker
            .execute(move || {
                started.send(()).unwrap();
                let _ = blocked.recv();
                Ok(None)
            })
            .await
    });

    running.await.unwrap();
    assert!(matches!(
        admission.execute(|| panic!("overloaded work ran")).await,
        Err(ApiError::SolveBusy)
    ));
    release.send(()).unwrap();
    assert!(matches!(task.await.unwrap(), Ok(None)));
    assert!(matches!(admission.execute(|| Ok(None)).await, Ok(None)));
}

#[tokio::test]
async fn aborting_request_does_not_release_slot_while_blocking_work_runs() {
    let admission = SolveAdmission::new(1);
    let worker = admission.clone();
    let (started, running) = oneshot::channel();
    let (release, blocked) = mpsc::channel();
    let task = tokio::spawn(async move {
        worker
            .execute(move || {
                started.send(()).unwrap();
                let _ = blocked.recv();
                Ok(None)
            })
            .await
    });

    running.await.unwrap();
    task.abort();
    assert!(matches!(task.await, Err(error) if error.is_cancelled()));
    assert!(matches!(
        admission
            .execute(|| panic!("cancelled request freed a running slot"))
            .await,
        Err(ApiError::SolveBusy)
    ));
    release.send(()).unwrap();
}
