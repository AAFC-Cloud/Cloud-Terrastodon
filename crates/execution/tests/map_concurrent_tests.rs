use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::join2;
use std::cell::Cell;
use std::cell::RefCell;
use std::future::Future;
use std::future::pending;
use std::future::poll_fn;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const DEADLOCK_GUARD: Duration = Duration::from_secs(5);

// Neither values nor errors need Clone; values may borrow the caller's inputs.
#[derive(Debug)]
struct BorrowedOutput<'a> {
    value: &'a str,
    invocation: usize,
}

#[derive(Debug)]
enum MapError {
    Operation(&'static str),
    Cancellation(CancellationError),
}

impl From<CancellationError> for MapError {
    fn from(value: CancellationError) -> Self {
        Self::Cancellation(value)
    }
}

async fn poll_until<F: Future>(mut future: Pin<&mut F>, reached: impl Fn() -> bool) {
    poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        if reached() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
}

#[tokio::test]
async fn bounds_active_work_and_returns_input_order_after_out_of_order_completion() {
    timeout(DEADLOCK_GUARD, async {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let labels = ["zero", "one", "two", "three"];
                let (mut releases, receivers): (Vec<_>, Vec<_>) = (0..4)
                    .map(|_| {
                        let (sender, receiver) = oneshot::channel();
                        (Some(sender), receiver)
                    })
                    .unzip();
                let admitted = &RefCell::new(Vec::new());
                let finished = &RefCell::new(Vec::new());
                let active = &Cell::new(0_usize);
                let peak = &Cell::new(0_usize);
                let mapped = labels
                    .iter()
                    .zip(receivers)
                    .enumerate()
                    .map_concurrent_scoped(
                        &scope,
                        NonZeroUsize::new(2).unwrap(),
                        |_child, (index, (label, released))| {
                            admitted.borrow_mut().push(index);
                            active.set(active.get() + 1);
                            peak.set(peak.get().max(active.get()));
                            async move {
                                released.await.unwrap();
                                active.set(active.get() - 1);
                                finished.borrow_mut().push(index);
                                Ok::<_, MapError>(BorrowedOutput {
                                    value: label,
                                    invocation: index,
                                })
                            }
                        },
                    );
                tokio::pin!(mapped);
                poll_until(mapped.as_mut(), || admitted.borrow().len() == 2).await;
                assert_eq!(*admitted.borrow(), [0, 1]);

                releases[1].take().unwrap().send(()).unwrap();
                poll_until(mapped.as_mut(), || admitted.borrow().len() == 3).await;
                releases[2].take().unwrap().send(()).unwrap();
                poll_until(mapped.as_mut(), || admitted.borrow().len() == 4).await;
                releases[0].take().unwrap().send(()).unwrap();
                poll_until(mapped.as_mut(), || finished.borrow().len() == 3).await;
                releases[3].take().unwrap().send(()).unwrap();

                let values = mapped.await?;
                assert_eq!(*finished.borrow(), [1, 2, 0, 3]);
                assert_eq!(peak.get(), 2);
                assert_eq!(active.get(), 0);
                assert_eq!(
                    values.iter().map(|value| value.value).collect::<Vec<_>>(),
                    labels,
                );
                Ok::<_, MapError>(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("bounded mapping must continue admitting work as slots become available");
}

#[tokio::test]
async fn send_future_supports_borrowed_outputs_and_a_mutable_non_sync_mapper() {
    let text = String::from("borrowed input");
    let borrowed_text = text.as_str();
    let values = CancellationToken::new()
        .with_async_scope(|scope| async move {
            let counter = Cell::new(0);
            let mut next = 0;
            let mapped = [borrowed_text, borrowed_text]
                .into_iter()
                .map_concurrent_scoped(
                    &scope,
                    NonZeroUsize::new(2).unwrap(),
                    move |_child, value| {
                        next += 1; // Requires FnMut rather than Fn.
                        counter.set(next); // Cell is Send but not Sync.
                        let invocation = counter.get();
                        async move { Ok::<_, MapError>(BorrowedOutput { value, invocation }) }
                    },
                );
            fn require_send<F: Future + Send>(future: F) -> F {
                future
            }
            require_send(mapped).await
        })
        .await
        .unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].value.as_ptr(), text.as_ptr());
    assert_eq!(values[1].value.as_ptr(), text.as_ptr());
    assert_eq!(values[0].invocation, 1);
    assert_eq!(values[1].invocation, 2);
}

#[tokio::test]
async fn error_stops_admission_cancels_optional_work_and_retains_observed_response_completion() {
    enum Job {
        ObservedResponse {
            started: oneshot::Sender<()>,
            released: oneshot::Receiver<()>,
        },
        Pending,
        Fail {
            started: oneshot::Receiver<()>,
            failed: oneshot::Sender<()>,
        },
        Queued,
    }

    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        token
            .with_async_scope(|scope| async move {
                let (started, has_started) = oneshot::channel();
                let (release, released) = oneshot::channel();
                let (failed, has_failed) = oneshot::channel();
                let factories = &Cell::new(0);
                let completed = &AtomicBool::new(false);
                let mapped = [
                    Job::ObservedResponse { started, released },
                    Job::Pending,
                    Job::Fail {
                        started: has_started,
                        failed,
                    },
                    Job::Queued,
                    Job::Queued,
                ]
                .into_iter()
                .map_concurrent_scoped(
                    &scope,
                    NonZeroUsize::new(3).unwrap(),
                    |child, job| {
                        factories.set(factories.get() + 1);
                        async move {
                            match job {
                                Job::ObservedResponse { started, released } => {
                                    child
                                        .finish_required(
                                            "retain observed synthetic response",
                                            |protected| async move {
                                                started.send(()).unwrap();
                                                released.await.unwrap();
                                                protected.bail_if_cancelled()?;
                                                completed.store(true, Ordering::SeqCst);
                                                Ok::<_, MapError>(())
                                            },
                                        )
                                        .await
                                }
                                Job::Pending => pending::<Result<(), MapError>>().await,
                                Job::Fail { started, failed } => {
                                    started.await.unwrap();
                                    failed.send(()).unwrap();
                                    Err(MapError::Operation("original failure"))
                                }
                                Job::Queued => {
                                    panic!("work after the failure must not be admitted")
                                }
                            }
                        }
                    },
                );
                tokio::pin!(mapped);
                tokio::select! {
                    biased;
                    result = &mut mapped => panic!("required completion was abandoned: {result:?}"),
                    result = has_failed => result.unwrap(),
                }
                assert!(!completed.load(Ordering::SeqCst));
                assert_eq!(factories.get(), 3);
                assert!(!scope.is_cancelled());
                release.send(()).unwrap();
                let error = mapped.await.unwrap_err();
                assert!(matches!(error, MapError::Operation("original failure")));
                assert!(completed.load(Ordering::SeqCst));
                assert_eq!(factories.get(), 3);
                // A failed map must not cancel unrelated work under its parent.
                scope
                    .cancellable_scope(|_| async { Ok::<_, MapError>(()) })
                    .await?;
                Ok::<_, MapError>(())
            })
            .await
            .unwrap();
        assert!(!token.is_cancelled());
    })
    .await
    .expect("failed mapping must cancel optional work and drain required completion");
}

#[tokio::test]
async fn automatic_parent_cancellation_drains_required_work_without_admitting_queued_jobs() {
    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        let (started, has_started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let factories = &Cell::new(0);
        let completed = &AtomicBool::new(false);
        let parent = token.cancellable_scope(|scope| async move {
            let mut protected_job = Some((started, released));
            (0..5)
                .map_concurrent_scoped(&scope, NonZeroUsize::new(2).unwrap(), |child, index| {
                    factories.set(factories.get() + 1);
                    let protected_job = (index == 0).then(|| protected_job.take().unwrap());
                    async move {
                        if let Some((started, released)) = protected_job {
                            child
                                .deferred("finish synthetic write", |protected| async move {
                                    started.send(()).unwrap();
                                    released.await.unwrap();
                                    protected.bail_if_cancelled()?;
                                    completed.store(true, Ordering::SeqCst);
                                    Ok::<_, MapError>(())
                                })
                                .await
                        } else {
                            pending::<Result<(), MapError>>().await
                        }
                    }
                })
                .await
        });
        tokio::pin!(parent);
        tokio::select! {
            biased;
            result = &mut parent => panic!("parent completed before cancellation: {result:?}"),
            result = has_started => result.unwrap(),
        }
        assert_eq!(factories.get(), 2);
        token.request_cancel("external parent cancellation");
        poll_until(parent.as_mut(), || true).await;
        assert!(!completed.load(Ordering::SeqCst));
        release.send(()).unwrap();
        let error = parent.await.unwrap_err();
        assert!(completed.load(Ordering::SeqCst));
        assert_eq!(factories.get(), 2);
        match error {
            MapError::Cancellation(error) => {
                assert!(error.to_string().contains("external parent cancellation"));
            }
            other => panic!("expected parent cancellation, got {other:?}"),
        }
    })
    .await
    .expect("automatic ancestors must retain the map's required descendants");
}

#[tokio::test]
async fn empty_input_does_not_invoke_the_mapper() {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let factories = Cell::new(0);
            let result = std::iter::empty::<()>()
                .map_concurrent_scoped(&scope, NonZeroUsize::MIN, |_, ()| {
                    factories.set(factories.get() + 1);
                    async { Ok::<_, MapError>(()) }
                })
                .await?;
            assert!(result.is_empty());
            assert_eq!(factories.get(), 0);
            Ok::<_, MapError>(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn errors_returned_as_data_do_not_stop_later_operations() {
    let outcomes = CancellationToken::new()
        .with_async_scope(|scope| async move {
            [Ok(10), Err("first operation failed"), Ok(20)]
                .into_iter()
                .map_concurrent_scoped(&scope, NonZeroUsize::MIN, |_, result| async move {
                    Ok::<_, MapError>(result)
                })
                .await
        })
        .await
        .unwrap();
    assert_eq!(outcomes, [Ok(10), Err("first operation failed"), Ok(20)]);
}

#[tokio::test]
async fn cancellation_before_admission_does_not_invoke_factories() {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let factories = Cell::new(0);
            let mapped =
                (0..3).map_concurrent_scoped(&scope, NonZeroUsize::new(2).unwrap(), |_, item| {
                    factories.set(factories.get() + 1);
                    async move { Ok::<_, MapError>(item) }
                });
            scope.request_cancel("cancel before the map is polled");
            assert!(matches!(mapped.await, Err(MapError::Cancellation(_))));
            assert_eq!(factories.get(), 0);
            Ok::<_, MapError>(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn outer_failure_drains_nested_map_write_without_starting_queued_writes() {
    timeout(DEADLOCK_GUARD, async {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let (started, has_started) = oneshot::channel();
                let (release, released) = oneshot::channel();
                let (failed, has_failed) = oneshot::channel();
                let factories = &Cell::new(0);
                let completed = &AtomicBool::new(false);
                let joined = join2(
                    &scope,
                    |child| async move {
                        let mut protected_write = Some((started, released));
                        (0..3)
                            .map_concurrent_scoped(&child, NonZeroUsize::MIN, |child, index| {
                                factories.set(factories.get() + 1);
                                assert_eq!(
                                    index, 0,
                                    "queued writes must not start after outer failure"
                                );
                                let (started, released) = protected_write.take().unwrap();
                                async move {
                                    child
                                        .deferred(
                                            "complete nested synthetic write",
                                            |protected| async move {
                                                started.send(()).unwrap();
                                                released.await.unwrap();
                                                protected.bail_if_cancelled()?;
                                                completed.store(true, Ordering::SeqCst);
                                                Ok::<_, MapError>(())
                                            },
                                        )
                                        .await
                                }
                            })
                            .await
                    },
                    |_| async move {
                        has_started.await.unwrap();
                        failed.send(()).unwrap();
                        Err::<(), _>(MapError::Operation("outer inventory failed"))
                    },
                );
                tokio::pin!(joined);
                tokio::select! {
                    biased;
                    result = &mut joined => panic!("nested write was abandoned: {result:?}"),
                    result = has_failed => result.unwrap(),
                }
                assert_eq!(factories.get(), 1);
                assert!(!completed.load(Ordering::SeqCst));
                release.send(()).unwrap();
                let error = joined.await.unwrap_err();
                assert!(matches!(
                    error,
                    MapError::Operation("outer inventory failed")
                ));
                assert!(completed.load(Ordering::SeqCst));
                assert_eq!(factories.get(), 1);
                assert!(!scope.is_cancelled());
                Ok::<_, MapError>(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("outer failure must drain a nested required write without admitting queued writes");
}

#[tokio::test]
async fn inherited_deferral_admits_remaining_work_in_the_protected_batch_after_cancellation() {
    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        let (started, has_started) = oneshot::channel();
        let (release, released) = oneshot::channel();
        let factories = &Cell::new(0);
        let completed = &RefCell::new(Vec::new());
        let parent = token.cancellable_scope(|scope| async move {
            scope
                .deferred(
                    "complete the entire synthetic batch",
                    |protected| async move {
                        let mut first_write = Some((started, released));
                        (0..3)
                            .map_concurrent_scoped(&protected, NonZeroUsize::MIN, |child, index| {
                                factories.set(factories.get() + 1);
                                let first_write = first_write.take();
                                async move {
                                    if let Some((started, released)) = first_write {
                                        started.send(()).unwrap();
                                        released.await.unwrap();
                                    }
                                    child.bail_if_cancelled()?;
                                    completed.borrow_mut().push(index);
                                    Ok::<_, MapError>(index)
                                }
                            })
                            .await
                    },
                )
                .await
        });
        tokio::pin!(parent);
        tokio::select! {
            biased;
            result = &mut parent => panic!("batch completed before its first write: {result:?}"),
            result = has_started => result.unwrap(),
        }
        assert_eq!(factories.get(), 1);
        token.request_cancel("finish the admitted required batch");
        poll_until(parent.as_mut(), || true).await;
        release.send(()).unwrap();
        let values = parent.await.unwrap();
        assert_eq!(values, [0, 1, 2]);
        assert_eq!(*completed.borrow(), [0, 1, 2]);
        assert_eq!(factories.get(), 3);
        assert!(token.is_cancelled());
    })
    .await
    .expect("inherited protection must apply to admission as well as active operations");
}
