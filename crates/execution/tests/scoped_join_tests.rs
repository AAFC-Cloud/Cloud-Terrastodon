use cloud_terrastodon_execution::join2;
use cloud_terrastodon_execution::join3;
use std::cell::Cell;
use std::future::Future;
use std::future::pending;
use std::future::poll_fn;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const DEADLOCK_GUARD: Duration = Duration::from_secs(5);

struct OwnedOnly(u32);

// Deliberately neither Clone nor convertible from eyre: joins preserve the
// caller's actual error, including an error from a later tuple position.
#[derive(Debug)]
enum JoinError {
    Operation(&'static str),
    Cancellation(CancellationError),
}

impl From<CancellationError> for JoinError {
    fn from(value: CancellationError) -> Self {
        Self::Cancellation(value)
    }
}

async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
    poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn borrowed_heterogeneous_outputs_need_neither_clone_nor_send() {
    let text = String::from("borrowed result");
    let local = Rc::new(Cell::new(7));
    let (borrowed, non_clone, non_send) = CancellationToken::new()
        .with_async_scope(async |scope| {
            join3(
                &scope,
                |_| async { Ok::<_, JoinError>(text.as_str()) },
                |_| async { Ok::<_, JoinError>(OwnedOnly(11)) },
                |_| async { Ok::<_, JoinError>(Rc::clone(&local)) },
            )
            .await
        })
        .await
        .unwrap();
    assert_eq!(borrowed.as_ptr(), text.as_ptr());
    assert_eq!(non_clone.0, 11);
    assert!(Rc::ptr_eq(&non_send, &local));
}

#[tokio::test]
async fn later_branch_failure_cancels_pending_work_without_cancelling_the_parent() {
    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        token
            .with_async_scope(|scope| async move {
                let (started, has_started) = oneshot::channel();
                let error = join2(
                    &scope,
                    |_| async {
                        started.send(()).unwrap();
                        pending::<Result<(), JoinError>>().await
                    },
                    |_| async {
                        has_started.await.unwrap();
                        Err::<String, _>(JoinError::Operation("original failure"))
                    },
                )
                .await
                .unwrap_err();
                assert!(matches!(error, JoinError::Operation("original failure")));
                assert!(!scope.is_cancelled());
                // Work outside the failed group still has a live owner/signal.
                scope
                    .cancellable_scope(|_| async { Ok::<_, JoinError>(()) })
                    .await?;
                Ok::<_, JoinError>(())
            })
            .await
            .unwrap();
        assert!(!token.is_cancelled());
    })
    .await
    .expect("the failed group must cancel its pending sibling");
}

#[tokio::test]
async fn first_failure_waits_for_required_completion_and_wins_over_cancellation_errors() {
    timeout(DEADLOCK_GUARD, async {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let completed = &AtomicBool::new(false);
                let failed = AtomicBool::new(false);
                let (started, has_started) = oneshot::channel();
                let (release, released) = oneshot::channel();
                let joined = join3(
                    &scope,
                    |child| async move {
                        child
                            .deferred("finish synthetic publication", |protected| async move {
                                started.send(()).unwrap();
                                released.await.unwrap();
                                protected.bail_if_cancelled()?;
                                completed.store(true, Ordering::SeqCst);
                                Ok::<_, JoinError>(())
                            })
                            .await
                    },
                    |_| async { pending::<Result<String, JoinError>>().await },
                    |_| async {
                        has_started.await.unwrap();
                        failed.store(true, Ordering::SeqCst);
                        Err::<usize, _>(JoinError::Operation("third branch failed"))
                    },
                );
                tokio::pin!(joined);
                assert_pending(joined.as_mut()).await;
                assert!(failed.load(Ordering::SeqCst));
                assert!(!completed.load(Ordering::SeqCst));
                release.send(()).unwrap();
                let error = joined.await.unwrap_err();
                assert!(completed.load(Ordering::SeqCst));
                assert!(matches!(error, JoinError::Operation("third branch failed")));
                Ok::<_, JoinError>(())
            })
            .await
            .unwrap();
    })
    .await
    .expect("required work must finish before returning the original error");
}

#[tokio::test]
async fn automatic_parent_cancellation_retains_active_required_completion() {
    timeout(DEADLOCK_GUARD, async {
        let token = CancellationToken::new();
        let completed = &AtomicBool::new(false);
        let (release, released) = oneshot::channel();
        let parent = token.cancellable_scope(|scope| async move {
            join2(
                &scope,
                |child| async move {
                    child
                        .deferred(
                            "finish observed synthetic response",
                            |protected| async move {
                                released.await.unwrap();
                                protected.bail_if_cancelled()?;
                                completed.store(true, Ordering::SeqCst);
                                Ok::<_, JoinError>(())
                            },
                        )
                        .await
                },
                |_| async { pending::<Result<(), JoinError>>().await },
            )
            .await
        });
        tokio::pin!(parent);
        assert_pending(parent.as_mut()).await;
        token.request_cancel("external parent cancellation");
        assert_pending(parent.as_mut()).await;
        assert!(!completed.load(Ordering::SeqCst));
        release.send(()).unwrap();
        let error = parent.await.unwrap_err();
        assert!(completed.load(Ordering::SeqCst));
        match error {
            JoinError::Cancellation(error) => {
                assert!(error.to_string().contains("external parent cancellation"));
            }
            other => panic!("expected cancellation after required completion, got {other:?}"),
        }
    })
    .await
    .expect("automatic ancestors must retain protected descendants");
}

#[tokio::test]
async fn cancellation_before_group_admission_does_not_invoke_factories() {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let called = Cell::new(false);
            let joined = join2(
                &scope,
                |_| {
                    called.set(true);
                    async { Ok::<_, JoinError>(()) }
                },
                |_| {
                    called.set(true);
                    async { Ok::<_, JoinError>(()) }
                },
            );
            scope.request_cancel("cancel before polling the group");
            assert!(matches!(joined.await, Err(JoinError::Cancellation(_))));
            assert!(!called.get());
            Ok::<_, JoinError>(())
        })
        .await
        .unwrap();
}
