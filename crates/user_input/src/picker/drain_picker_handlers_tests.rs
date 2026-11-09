use super::drain_picker_handlers::drain_picker_handlers;
use super::event_handler::EventHandler;
use super::handler_tasks::HandlerTasks;
use super::picker_event::PickerEvent;
use crate::TerminalBackend;
use crate::TerminalControl;
use crate::TerminalCoordinator;
use crate::apply_terminal_control;
use futures::FutureExt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use tokio::sync::Notify;
use tokio::sync::mpsc;

#[derive(Default)]
struct TestBackend {
    active: bool,
    suspended: usize,
    resumed: usize,
}

impl TerminalBackend for TestBackend {
    fn is_active(&self) -> bool {
        self.active
    }

    fn suspend(&mut self) -> eyre::Result<()> {
        self.active = false;
        self.suspended += 1;
        Ok(())
    }

    fn resume(&mut self) -> eyre::Result<()> {
        self.active = true;
        self.resumed += 1;
        Ok(())
    }
}

#[tokio::test]
async fn shutdown_retains_nested_handler_until_required_work_and_terminal_resume_finish() {
    drain_nested_handler(false).await;
}

#[tokio::test]
async fn poison_retains_nested_required_work_without_waiting_for_an_impossible_resume() {
    drain_nested_handler(true).await;
}

async fn drain_nested_handler(poisoned: bool) {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let coordinator = TerminalCoordinator::new()?;
            let mut guard = coordinator.acquire().await?;
            let mut backend = TestBackend {
                active: true,
                ..Default::default()
            };
            let acquired = Arc::new(Notify::new());
            let release = Arc::new(Notify::new());
            let cancellation_observed = Arc::new(AtomicBool::new(false));
            let finished = Arc::new(AtomicBool::new(false));
            let handler = EventHandler::<()>::new({
                let coordinator = coordinator.clone();
                let acquired = Arc::clone(&acquired);
                let release = Arc::clone(&release);
                let cancellation_observed = Arc::clone(&cancellation_observed);
                let finished = Arc::clone(&finished);
                move |scope, _, _| {
                    let coordinator = coordinator.clone();
                    let acquired = Arc::clone(&acquired);
                    let release = Arc::clone(&release);
                    let cancellation_observed = Arc::clone(&cancellation_observed);
                    let finished = Arc::clone(&finished);
                    async move {
                        scope
                            .deferred("restore nested terminal after completion", async |owner| {
                                let nested = coordinator.acquire().await?;
                                acquired.notify_one();
                                owner.cancelled().await;
                                cancellation_observed.store(true, Ordering::SeqCst);
                                release.notified().await;
                                let result = nested.release().await;
                                finished.store(true, Ordering::SeqCst);
                                result
                            })
                            .await
                    }
                }
            });
            let (sender, _receiver) = mpsc::unbounded_channel();
            let mut handlers = HandlerTasks::new(scope);
            handlers.spawn(&[handler], PickerEvent::InitialLoad, true, &sender);

            tokio::time::timeout(Duration::from_secs(1), async {
                loop {
                    tokio::select! {
                        () = acquired.notified() => break,
                        control = guard.next_control() => {
                            apply_terminal_control(control.unwrap(), &mut guard, &mut backend)?;
                        }
                        completed = handlers.next() => {
                            panic!("nested handler finished before cancellation: {completed:?}");
                        }
                    }
                }
                eyre::Ok(())
            })
            .await??;
            assert!(!backend.active);
            assert_eq!(backend.suspended, 1);
            assert!(!finished.load(Ordering::SeqCst));

            if poisoned {
                coordinator.poison("synthetic terminal failure");
                assert!(matches!(
                    guard.next_control().await,
                    Some(TerminalControl::Poisoned { .. })
                ));
            }

            let mut draining = Box::pin(drain_picker_handlers(
                &mut handlers,
                &mut guard,
                &mut backend,
            ));
            assert!(draining.as_mut().now_or_never().is_none());
            assert!(cancellation_observed.load(Ordering::SeqCst));
            assert!(!finished.load(Ordering::SeqCst));

            release.notify_one();
            let result = tokio::time::timeout(Duration::from_secs(1), draining).await?;
            assert_eq!(result.is_err(), poisoned);
            assert!(finished.load(Ordering::SeqCst));
            assert!(handlers.is_empty());
            assert_eq!(backend.active, !poisoned);
            assert_eq!(backend.resumed, usize::from(!poisoned));
            guard.release().await?;
            assert!(!coordinator.activity().is_active());
            eyre::Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn already_consumed_poison_does_not_wait_for_inactive_backend_to_resume() {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let coordinator = TerminalCoordinator::new()?;
            let mut guard = coordinator.acquire().await?;
            let mut backend = TestBackend::default();
            coordinator.poison("synthetic backend restoration failure");
            assert!(matches!(
                guard.next_control().await,
                Some(TerminalControl::Poisoned { .. })
            ));
            let mut handlers = HandlerTasks::new(scope);
            let result = tokio::time::timeout(
                Duration::from_secs(1),
                drain_picker_handlers(&mut handlers, &mut guard, &mut backend),
            )
            .await?;
            assert!(result.is_err());
            assert!(!backend.active);
            assert_eq!(backend.resumed, 0);
            guard.release().await?;
            assert!(!coordinator.activity().is_active());
            eyre::Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn poison_after_drain_starts_rejects_a_queued_resume_without_touching_the_backend() {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let coordinator = TerminalCoordinator::new()?;
            let mut guard = coordinator.acquire().await?;
            let mut backend = TestBackend {
                active: true,
                ..Default::default()
            };
            let mut acquiring = Box::pin(coordinator.acquire());
            let suspend = tokio::select! {
                control = guard.next_control() => control.unwrap(),
                nested = &mut acquiring => {
                    drop(nested?);
                    panic!("nested frame acquired before suspension");
                }
            };
            apply_terminal_control(suspend, &mut guard, &mut backend)?;
            let nested = acquiring.await?;
            let mut handlers = HandlerTasks::new(scope);
            let mut draining = Box::pin(drain_picker_handlers(
                &mut handlers,
                &mut guard,
                &mut backend,
            ));
            // The drain starts while healthy and waits for the nested owner.
            assert!(draining.as_mut().now_or_never().is_none());
            let mut releasing = Box::pin(nested.release());
            assert!(releasing.as_mut().now_or_never().is_none());
            tokio::task::yield_now().await;
            // Resume may already be queued, but poisoning invalidates it before
            // the drain is polled again and touches the physical backend.
            coordinator.poison("synthetic failure after resume was requested");
            let (drained, released) = tokio::time::timeout(Duration::from_secs(1), async {
                tokio::join!(draining, releasing)
            })
            .await?;
            assert!(drained.is_err());
            assert!(released.is_err());
            assert!(!backend.active);
            assert_eq!(backend.resumed, 0);
            guard.release().await?;
            assert!(!coordinator.activity().is_active());
            eyre::Ok(())
        })
        .await
        .unwrap();
}
