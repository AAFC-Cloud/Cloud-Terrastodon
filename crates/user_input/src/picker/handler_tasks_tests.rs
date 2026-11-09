use super::event_handler::EventHandler;
use super::handler_tasks::HandlerTasks;
use super::picker_event::PickerEvent;
use super::picker_event_state::PickerEventState;
use super::picker_tui::new_nucleo;
use super::picker_tui::process_candidate_message;
use crate::Choice;
use futures::FutureExt;
use rustc_hash::FxHashSet;
use std::cell::Cell;
use std::future::pending;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use tokio::sync::Notify;
use tokio::sync::mpsc;

#[tokio::test]
async fn reload_keeps_required_old_work_and_isolates_its_candidates_errors_and_counts() {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let old_release = Arc::new(Notify::new());
            let new_release = Arc::new(Notify::new());
            let old_finished = Arc::new(AtomicBool::new(false));
            let handlers = [EventHandler::new({
                let old_release = Arc::clone(&old_release);
                let new_release = Arc::clone(&new_release);
                let old_finished = Arc::clone(&old_finished);
                move |scope, event, sink| {
                    let old_release = Arc::clone(&old_release);
                    let new_release = Arc::clone(&new_release);
                    let old_finished = Arc::clone(&old_finished);
                    async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            scope
                                .deferred("finish observed response", async |_completion| {
                                    old_release.notified().await;
                                    old_finished.store(true, Ordering::SeqCst);
                                    sink.push([Choice {
                                        key: "same selection".to_owned(),
                                        value: "obsolete",
                                    }])?;
                                    eyre::bail!("obsolete request failed after completion");
                                })
                                .await
                        } else {
                            sink.push([Choice {
                                key: "same selection".to_owned(),
                                value: "current",
                            }])?;
                            new_release.notified().await;
                            Ok(())
                        }
                    }
                }
            })];
            let (sender, mut receiver) = mpsc::unbounded_channel();
            let mut tasks = HandlerTasks::new(scope.clone());
            tasks.spawn(&handlers, PickerEvent::InitialLoad, true, &sender);
            assert!(tasks.next().now_or_never().is_none());

            tasks.restart();
            tasks.spawn(
                &handlers,
                PickerEvent::ReloadRequested(Arc::from("")),
                true,
                &sender,
            );
            assert!(tasks.next().now_or_never().is_none());
            assert!(!old_finished.load(Ordering::SeqCst));
            let current = receiver.try_recv().unwrap();
            assert_eq!(current.generation, tasks.generation());
            assert_eq!(tasks.pending(), 1);
            assert_eq!(tasks.startup(), 1);

            old_release.notify_one();
            tokio::time::timeout(Duration::from_secs(1), tasks.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(old_finished.load(Ordering::SeqCst));
            assert_eq!(tasks.pending(), 1);
            assert_eq!(tasks.startup(), 1);
            let obsolete = receiver.try_recv().unwrap();
            assert_ne!(obsolete.generation, tasks.generation());

            let mut state = PickerEventState::default();
            state.reload();
            let mut nucleo = new_nucleo();
            let mut warned = FxHashSet::default();
            for message in [current, obsolete] {
                process_candidate_message(
                    message,
                    tasks.generation(),
                    &mut state,
                    &mut nucleo,
                    &mut warned,
                );
            }
            assert_eq!(state.candidates.get("same selection"), Some(&"current"));

            new_release.notify_one();
            tasks.next().await.unwrap()?;
            assert!(tasks.is_empty());
            assert_eq!(tasks.pending(), 0);
            assert_eq!(tasks.startup(), 0);
            assert!(!scope.is_cancelled());
            eyre::Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn shutdown_cancels_optional_work_and_drains_required_completion() {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let release = Arc::new(Notify::new());
            let finished = Arc::new(AtomicBool::new(false));
            let required = EventHandler::<()>::new({
                let release = Arc::clone(&release);
                let finished = Arc::clone(&finished);
                move |scope, _, _| {
                    let release = Arc::clone(&release);
                    let finished = Arc::clone(&finished);
                    async move {
                        scope
                            .deferred("retain required completion", async |_completion| {
                                release.notified().await;
                                finished.store(true, Ordering::SeqCst);
                                eyre::Ok(())
                            })
                            .await
                    }
                }
            });
            let optional = EventHandler::new(async |_, _, _| pending::<eyre::Result<()>>().await);
            let (sender, _receiver) = mpsc::unbounded_channel();
            let mut tasks = HandlerTasks::new(scope.clone());
            tasks.spawn(
                &[required, optional],
                PickerEvent::InitialLoad,
                true,
                &sender,
            );
            assert!(tasks.next().now_or_never().is_none());

            tasks.cancel("selection accepted");
            tasks.next().await.unwrap()?;
            assert!(tasks.next().now_or_never().is_none());
            assert!(!finished.load(Ordering::SeqCst));
            assert!(!scope.is_cancelled());

            release.notify_one();
            tasks.next().await.unwrap()?;
            assert!(tasks.is_empty());
            assert!(finished.load(Ordering::SeqCst));

            let forbidden = EventHandler::new(|_, _, _| pending::<eyre::Result<()>>());
            tasks.spawn(&[forbidden], PickerEvent::QueryCleared, false, &sender);
            assert!(tasks.is_empty());
            eyre::Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn handler_errors_and_factory_or_future_panics_retain_required_siblings() {
    for failure in ["error", "factory panic", "future panic"] {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let release = Arc::new(Notify::new());
                let finished = Arc::new(AtomicBool::new(false));
                let required = EventHandler::<()>::new({
                    let release = Arc::clone(&release);
                    let finished = Arc::clone(&finished);
                    move |scope, _, _| {
                        let release = Arc::clone(&release);
                        let finished = Arc::clone(&finished);
                        async move {
                            scope
                                .deferred("complete sibling response", async |_completion| {
                                    release.notified().await;
                                    finished.store(true, Ordering::SeqCst);
                                    eyre::Ok(())
                                })
                                .await
                        }
                    }
                });
                let (sender, _receiver) = mpsc::unbounded_channel();
                let mut tasks = HandlerTasks::new(scope.clone());
                tasks.spawn(&[required], PickerEvent::InitialLoad, true, &sender);
                assert!(tasks.next().now_or_never().is_none());
                let failing = EventHandler::new(move |_, _, _| {
                    assert_ne!(failure, "factory panic", "synthetic factory panic");
                    async move {
                        assert_ne!(failure, "future panic", "synthetic future panic");
                        eyre::bail!("synthetic handler error");
                    }
                });
                tasks.spawn(&[failing], PickerEvent::InitialLoad, true, &sender);
                let error = tasks.next().await.unwrap().unwrap_err();
                assert!(error.to_string().contains("synthetic"));
                assert!(!finished.load(Ordering::SeqCst));
                assert!(tasks.next().now_or_never().is_none());
                assert!(!scope.is_cancelled());

                release.notify_one();
                tasks.next().await.unwrap()?;
                assert!(tasks.is_empty());
                assert!(finished.load(Ordering::SeqCst));
                eyre::Ok(())
            })
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn cancelled_unpolled_handlers_never_invoke_the_non_sync_factory() {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let calls = Arc::new(AtomicUsize::new(0));
            let handlers = [EventHandler::<()>::new({
                let calls = Arc::clone(&calls);
                let local_calls = Cell::new(0);
                move |_, _, _| {
                    local_calls.set(local_calls.get() + 1);
                    calls.store(local_calls.get(), Ordering::SeqCst);
                    async { eyre::Ok(()) }
                }
            })];
            let (sender, _receiver) = mpsc::unbounded_channel();
            let mut tasks = HandlerTasks::new(scope);
            tasks.spawn(&handlers, PickerEvent::InitialLoad, true, &sender);
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            tasks.restart();
            tasks.spawn(&handlers, PickerEvent::QueryCleared, false, &sender);
            while let Some(result) = tasks.next().await {
                result?;
            }
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(tasks.pending(), 0);
            assert_eq!(tasks.startup(), 0);
            eyre::Ok(())
        })
        .await
        .unwrap();
}
