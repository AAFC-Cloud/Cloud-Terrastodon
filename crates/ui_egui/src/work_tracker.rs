use crate::work_handle::WorkHandle;
use cloud_terrastodon_command::CancellationReportExt;
use eyre::Report;
use futures::FutureExt;
use std::cell::RefCell;
use tokio::task::JoinError;
use tracing::error;
use tracing::info;

#[derive(Debug, Default)]
pub struct WorkTracker {
    // UI-thread ownership permits reaping completed tasks without blocking a frame.
    remaining_work: RefCell<Vec<WorkHandle>>,
    first_error: RefCell<Option<Report>>,
}

impl WorkTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track(&self, work: WorkHandle) {
        self.remaining_work.borrow_mut().push(work);
    }

    pub fn prune(&self) {
        self.remaining_work.borrow_mut().retain_mut(|work| {
            // Poll a borrowed handle: a pending outcome stays owned by the
            // tracker, and a completed panic/error is not silently discarded.
            let Some(result) = (&mut work.join_handle).now_or_never() else {
                return true;
            };
            self.record_completion(&work.description, result);
            false
        });
    }

    fn record_completion(&self, description: &str, result: Result<eyre::Result<()>, JoinError>) {
        let result = match result {
            Ok(result) => result,
            Err(error) => Err(Report::new(error)),
        };
        if let Err(error) = result {
            if error.is_cancellation() {
                info!(%description, "Background work acknowledged cancellation");
                return;
            }
            let error = error.wrap_err(format!("Background work {description:?} failed"));
            error!(
                ?error,
                "Background work failed; remaining tasks will still be drained"
            );
            let mut first_error = self.first_error.borrow_mut();
            if first_error.is_none() {
                *first_error = Some(error);
            }
        }
    }

    /// Drain after the GUI has stopped enqueuing work and cancelled its executor.
    /// The caller protects this cleanup while the execution scope remains open.
    pub async fn finish(&self) -> eyre::Result<()> {
        let mut remaining_work = self.remaining_work.take();
        while let Some(work) = remaining_work.pop() {
            info!(description = %work.description, remaining = remaining_work.len(), "Draining background work");
            self.record_completion(&work.description, work.join_handle.await);
        }
        match self.first_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkTracker;
    use crate::work_handle::WorkHandle;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn panic_does_not_skip_another_background_write() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let tracker = WorkTracker::new();
            let (release, released) = oneshot::channel();
            let complete = Arc::new(AtomicBool::new(false));
            let complete_by_task = Arc::clone(&complete);
            tracker.track(WorkHandle {
                description: "write".into(),
                join_handle: tokio::spawn(async move {
                    released.await.unwrap();
                    complete_by_task.store(true, Ordering::SeqCst);
                    eyre::Ok(())
                }),
            });
            tracker.track(WorkHandle {
                description: "panic".into(),
                join_handle: tokio::spawn(async {
                    panic!("synthetic background panic");
                    #[allow(unreachable_code)]
                    eyre::Ok(())
                }),
            });
            while tracker.remaining_work.borrow().len() > 1 {
                tokio::task::yield_now().await;
                tracker.prune();
            }
            let finish = tracker.finish();
            tokio::pin!(finish);
            std::future::poll_fn(|cx| {
                assert!(finish.as_mut().poll(cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            assert!(!complete.load(Ordering::SeqCst));
            release.send(()).unwrap();
            let error = finish.await.unwrap_err();
            assert!(format!("{error:#}").contains("synthetic background panic"));
            assert!(complete.load(Ordering::SeqCst));
        })
        .await
        .expect("a GUI task panic must not abandon outstanding writes");
    }
}
