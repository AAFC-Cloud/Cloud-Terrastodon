use super::AppWorkState;
use super::AppWorkTracker;
use super::Loadable;
use super::LoadableWorkBuilder;
use eyre::Report;
use std::future::pending;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use teamy_tracing::capture::CaptureLayer;
use teamy_tracing::capture::LogBuffer;
use teamy_tracing::capture::LogValue;
use teamy_tracing::capture::format_record;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing_subscriber::layer::SubscriberExt;

const DEADLOCK_GUARD: Duration = Duration::from_secs(5);

struct NotifyDrop(Option<oneshot::Sender<()>>);

impl Drop for NotifyDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn cancellation_before_the_first_poll_does_not_invoke_the_factory() {
    timeout(DEADLOCK_GUARD, async {
        let parent = CancellationToken::new();
        parent
            .with_async_scope(async |scope| {
                let tracker = AppWorkTracker::new();
                let invoked = Arc::new(AtomicBool::new(false));
                let invoked_by_factory = Arc::clone(&invoked);
                tracker.spawn(
                    &scope,
                    &Handle::current(),
                    "queued work".into(),
                    move |_child| {
                        invoked_by_factory.store(true, Ordering::SeqCst);
                        async { eyre::Ok(()) }
                    },
                )?;

                // No yield occurs between admission and cancellation, so this
                // current-thread runtime cannot have polled the queued task.
                parent.request_cancel("cancel before the first poll");
                tracker.finish(&scope).await?;
                assert!(!invoked.load(Ordering::SeqCst));
                eyre::Ok(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("cancelled queued work must drain without invoking its factory");
}

#[tokio::test]
async fn shutdown_cancels_optional_work_and_waits_for_required_completion() {
    timeout(DEADLOCK_GUARD, async {
        let parent = CancellationToken::new();
        parent.with_async_scope(async |scope| {
            let tracker = AppWorkTracker::new();
            let (optional_started, optional_ready) = oneshot::channel();
            let (optional_dropped, observed_drop) = oneshot::channel();
            tracker.spawn(&scope, &Handle::current(), "optional".into(), async move |_child| {
                let _drop = NotifyDrop(Some(optional_dropped));
                optional_started.send(()).unwrap();
                pending::<eyre::Result<()>>().await
            })?;
            let (required_started, required_ready) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let completed = Arc::new(AtomicBool::new(false));
            let completed_by_task = Arc::clone(&completed);
            tracker.spawn(&scope, &Handle::current(), "required".into(), async move |child| {
                child.deferred("observed response needs persistence", async move |protected| {
                    required_started.send(()).unwrap();
                    released.await.unwrap();
                    protected.bail_if_cancelled()?;
                    completed_by_task.store(true, Ordering::SeqCst);
                    eyre::Ok(())
                }).await
            })?;
            optional_ready.await.unwrap();
            required_ready.await.unwrap();

            let finish = tracker.clone().finish(&scope);
            tokio::pin!(finish);
            tokio::select! {
                biased;
                result = &mut finish => panic!("shutdown returned before required completion: {result:?}"),
                () = async { observed_drop.await.unwrap(); } => {}
            }
            assert!(!completed.load(Ordering::SeqCst));
            assert!(!parent.is_cancelled(), "shutdown must not cancel an unrelated parent");
            assert!(tracker.spawn(&scope, &Handle::current(), "late".into(), async |_child| {
                panic!("closed admission must not invoke a new factory");
                #[allow(unreachable_code)]
                eyre::Ok(())
            }).is_err());
            release.send(()).unwrap();
            finish.await?;
            assert!(completed.load(Ordering::SeqCst));
            eyre::Ok(())
        }).await.unwrap();
    }).await.expect("shutdown must cancel optional work and drain required work");
}

#[tokio::test]
async fn task_errors_and_panics_survive_until_every_other_task_is_drained() {
    timeout(DEADLOCK_GUARD, async {
        for (panic, prune) in [(false, false), (false, true), (true, false), (true, true)] {
            CancellationToken::new()
                .with_async_scope(async |scope| {
                    let tracker = AppWorkTracker::new();
                    let (started, ready) = oneshot::channel();
                    let (release, released) = oneshot::channel();
                    let completed = Arc::new(AtomicBool::new(false));
                    let completed_by_task = Arc::clone(&completed);
                    tracker.spawn(
                        &scope,
                        &Handle::current(),
                        "required sibling".into(),
                        async move |child| {
                            child
                                .deferred("finish admitted work", async move |_protected| {
                                    started.send(()).unwrap();
                                    released.await.unwrap();
                                    completed_by_task.store(true, Ordering::SeqCst);
                                    eyre::Ok(())
                                })
                                .await
                        },
                    )?;
                    ready.await.unwrap();
                    let (failed, failure_ready) = oneshot::channel();
                    tracker.spawn(
                        &scope,
                        &Handle::current(),
                        "failure".into(),
                        async move |_child| {
                            failed.send(()).unwrap();
                            assert!(!panic, "synthetic task panic");
                            eyre::bail!("synthetic task error");
                        },
                    )?;
                    failure_ready.await.unwrap();
                    if prune {
                        while tracker.len()? > 1 {
                            tokio::task::yield_now().await;
                            tracker.prune()?;
                        }
                    }
                    let finish = tracker.finish(&scope);
                    tokio::pin!(finish);
                    std::future::poll_fn(|cx| {
                        assert!(finish.as_mut().poll(cx).is_pending());
                        std::task::Poll::Ready(())
                    })
                    .await;
                    assert!(!completed.load(Ordering::SeqCst));
                    release.send(()).unwrap();
                    let error = finish.await.unwrap_err();
                    assert!(completed.load(Ordering::SeqCst));
                    let diagnostic = format!("{error:#}");
                    assert!(diagnostic.contains(if panic {
                        "synthetic task panic"
                    } else {
                        "synthetic task error"
                    }));
                    eyre::Ok(())
                })
                .await
                .unwrap();
        }
    })
    .await
    .expect("a pruned error must not bypass draining the remaining tasks");
}

#[tokio::test]
async fn loadable_work_receives_a_linked_scope_and_publishes_required_success() {
    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        token
            .with_async_scope(async |scope| {
                let mut app = AppWorkState::<Loadable<usize>>::new();
                let mut state = Loadable::NotLoaded;
                let (started, ready) = oneshot::channel();
                let (release, released) = oneshot::channel();
                LoadableWorkBuilder::new()
                    .description("required load")
                    .setter(|state, result| *state = result)
                    .work(async move |child| {
                        child
                            .deferred("keep the decoded response", async move |protected| {
                                started.send(()).unwrap();
                                released.await.unwrap();
                                protected.bail_if_cancelled()?;
                                eyre::Ok(42)
                            })
                            .await
                    })
                    .build()?
                    .enqueue(
                        &app.work_tracker,
                        app.work_response_sender.clone(),
                        &mut state,
                        &scope,
                    )?;
                assert!(matches!(state, Loadable::Loading { .. }));
                ready.await.unwrap();
                token.request_cancel("parent shutdown");
                let finish = app.work_tracker.clone().finish(&scope);
                tokio::pin!(finish);
                std::future::poll_fn(|cx| {
                    assert!(finish.as_mut().poll(cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                release.send(()).unwrap();
                finish.await?;
                app.handle_messages(&mut state)?;
                assert_eq!(state.as_loaded(), Some(&42));
                eyre::Ok(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("required publication must survive parent cancellation");
}

#[tokio::test(flavor = "current_thread")]
async fn failed_loadable_work_shares_its_report_with_logging_and_application_state() {
    let buffer = LogBuffer::new();
    let subscriber =
        tracing_subscriber::registry().with(CaptureLayer::new(&buffer, Arc::new(|| true), |_| {}));
    // The current-thread runtime keeps spawned work within this thread-local
    // subscriber without installing a process-global logger for other tests.
    let _subscriber = tracing::subscriber::set_default(subscriber);

    timeout(DEADLOCK_GUARD, async {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let mut app = AppWorkState::<Loadable<usize>>::new();
                let mut state = Loadable::NotLoaded;
                LoadableWorkBuilder::new()
                    .description("synthetic failed load")
                    .setter(|state, result| *state = result)
                    .work(async |_child| {
                        Err::<usize, _>(Report::new(Error::new(
                            ErrorKind::PermissionDenied,
                            "\x1b[91msynthetic offline failure\x1b[0m",
                        )))
                    })
                    .build()?
                    .enqueue(
                        &app.work_tracker,
                        app.work_response_sender.clone(),
                        &mut state,
                        &scope,
                    )?;

                while !matches!(state, Loadable::Failed { .. }) {
                    app.handle_messages(&mut state)?;
                    tokio::task::yield_now().await;
                }
                app.work_tracker.finish(&scope).await?;

                let Loadable::Failed { error, .. } = state else {
                    panic!("failed work must publish its report to application state");
                };
                assert_eq!(
                    error.report.downcast_ref::<Error>().unwrap().kind(),
                    ErrorKind::PermissionDenied,
                );

                let records = buffer.records_since(&mut 0);
                let record = records
                    .iter()
                    .find(|record| record.fields.iter().any(|(name, _)| name == "error"))
                    .expect("failed work must emit an error event");
                let (_, LogValue::Report(logged)) = record
                    .fields
                    .iter()
                    .find(|(name, _)| name == "error")
                    .expect("the failure event must have an error field")
                else {
                    panic!("worker failures must retain a typed diagnostic field");
                };
                assert!(Arc::ptr_eq(&logged.report, &error.report));

                let terminal = format_record(record);
                let plain = error.to_string();
                for presentation in [&terminal, &plain] {
                    assert!(presentation.contains("synthetic offline failure"));
                    assert!(presentation.contains("Error encountered in worker thread"));
                    assert!(!presentation.contains("\\u{1b}["));
                }
                assert!(!plain.contains('\x1b'));
                eyre::Ok(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("failed work must publish and drain without losing its diagnostic");
}

#[tokio::test]
async fn closed_receiver_ignores_success_but_reports_failure_after_draining_siblings() {
    timeout(DEADLOCK_GUARD, async {
        for failed in [false, true] {
            CancellationToken::new()
                .with_async_scope(async |scope| {
                    let mut app = AppWorkState::<Loadable<usize>>::new();
                    let mut state = Loadable::NotLoaded;
                    app.work_response_receiver.close();
                    let (started, ready) = oneshot::channel();
                    let (release, released) = oneshot::channel();
                    let completed = Arc::new(AtomicBool::new(false));
                    let completed_by_task = Arc::clone(&completed);
                    app.work_tracker.spawn(
                        &scope,
                        &Handle::current(),
                        "required completion".into(),
                        async move |child| {
                            child
                                .deferred("finish already-started work", async move |_protected| {
                                    started.send(()).unwrap();
                                    released.await.unwrap();
                                    completed_by_task.store(true, Ordering::SeqCst);
                                    eyre::Ok(())
                                })
                                .await
                        },
                    )?;
                    ready.await.unwrap();
                    LoadableWorkBuilder::new()
                        .description("delivery")
                        .setter(|state, result| *state = result)
                        .work(async move |_child| {
                            if failed {
                                eyre::bail!("synthetic load failure");
                            }
                            eyre::Ok(1)
                        })
                        .build()?
                        .enqueue(
                            &app.work_tracker,
                            app.work_response_sender.clone(),
                            &mut state,
                            &scope,
                        )?;
                    while app.work_tracker.len()? > 1 {
                        tokio::task::yield_now().await;
                        app.work_tracker.prune()?;
                    }
                    let finish = app.work_tracker.finish(&scope);
                    tokio::pin!(finish);
                    std::future::poll_fn(|cx| {
                        assert!(finish.as_mut().poll(cx).is_pending());
                        std::task::Poll::Ready(())
                    })
                    .await;
                    release.send(()).unwrap();
                    let result = finish.await;
                    if failed {
                        let error = result.unwrap_err();
                        assert!(format!("{error:#}").contains("UI receiver closed"));
                    } else {
                        result?;
                    }
                    assert!(completed.load(Ordering::SeqCst));
                    eyre::Ok(())
                })
                .await
                .unwrap();
        }
    })
    .await
    .expect("a closed receiver must not discard required work");
}
