use super::event_handler::EventHandler;
use super::handler_tasks::HandlerTasks;
use super::picker_event::PickerEvent;
use super::picker_session::run_picker_session;
use futures::FutureExt;
use std::future::pending;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use teamy_cancellation::CancellationToken;
use tokio::sync::Notify;
use tokio::sync::mpsc;

struct RecordDrop(Arc<AtomicBool>);

impl Drop for RecordDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn automatic_parent_retains_terminal_cleanup_without_shielding_optional_handlers() {
    let cancellation = CancellationToken::new();
    let optional_dropped = Arc::new(AtomicBool::new(false));
    let required_started = Arc::new(AtomicBool::new(false));
    let required_finished = Arc::new(AtomicBool::new(false));
    let required_release = Arc::new(Notify::new());
    let cleanup_started = AtomicBool::new(false);
    let cleanup_finished = AtomicBool::new(false);
    let cleanup_release = Notify::new();

    let mut picker = Box::pin(cancellation.cancellable_scope(async |parent| {
        run_picker_session(&parent, async |owner| {
            let optional = EventHandler::<()>::new({
                let optional_dropped = Arc::clone(&optional_dropped);
                move |_, _, _| {
                    let optional_dropped = Arc::clone(&optional_dropped);
                    async move {
                        let _drop = RecordDrop(optional_dropped);
                        pending::<eyre::Result<()>>().await
                    }
                }
            });
            let required = EventHandler::new({
                let required_started = Arc::clone(&required_started);
                let required_finished = Arc::clone(&required_finished);
                let required_release = Arc::clone(&required_release);
                move |scope, _, _| {
                    let required_started = Arc::clone(&required_started);
                    let required_finished = Arc::clone(&required_finished);
                    let required_release = Arc::clone(&required_release);
                    async move {
                        scope
                            .deferred("finish observed response", async |_completion| {
                                required_started.store(true, Ordering::SeqCst);
                                required_release.notified().await;
                                required_finished.store(true, Ordering::SeqCst);
                                eyre::Ok(())
                            })
                            .await
                    }
                }
            });
            let mut tasks = HandlerTasks::new(owner);
            let (sender, _receiver) = mpsc::unbounded_channel();
            tasks.spawn(
                &[optional, required],
                PickerEvent::InitialLoad,
                true,
                &sender,
            );
            while let Some(result) = tasks.next().await {
                result?;
            }

            // Terminal restoration can itself suspend (for example, while a
            // nested owner acknowledges its release). Ancestors must retain it
            // even after the last handler's required completion ends.
            cleanup_started.store(true, Ordering::SeqCst);
            cleanup_release.notified().await;
            cleanup_finished.store(true, Ordering::SeqCst);
            eyre::Ok(())
        })
        .await
    }));
    assert!(picker.as_mut().now_or_never().is_none());
    assert!(required_started.load(Ordering::SeqCst));
    assert!(!optional_dropped.load(Ordering::SeqCst));

    cancellation.request_cancel("application is shutting down");
    assert!(picker.as_mut().now_or_never().is_none());
    assert!(optional_dropped.load(Ordering::SeqCst));
    assert!(!required_finished.load(Ordering::SeqCst));
    assert!(!cleanup_started.load(Ordering::SeqCst));

    required_release.notify_one();
    assert!(picker.as_mut().now_or_never().is_none());
    assert!(required_finished.load(Ordering::SeqCst));
    assert!(cleanup_started.load(Ordering::SeqCst));
    assert!(!cleanup_finished.load(Ordering::SeqCst));

    cleanup_release.notify_one();
    picker.await.unwrap();
    assert!(cleanup_finished.load(Ordering::SeqCst));
}
