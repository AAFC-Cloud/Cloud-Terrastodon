#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::cell::Cell;
use std::future::Future;
use std::future::poll_fn;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use tokio::sync::Barrier;
use tokio::sync::oneshot;
use tokio::time::timeout;

const AUTHENTICATION: AuthenticationSelection = AuthenticationSelection::Anonymous;
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

async fn assert_pending<F: Future + ?Sized>(mut future: Pin<&mut F>) {
    poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn outputs_can_borrow_and_remain_owned_without_sync_or_send_observers() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let text = String::from("borrowed uncached result");
            let (borrowed, closed) = executor
                .uncached_work(&AUTHENTICATION, |child| {
                    let text = &text;
                    async move { Ok((text.as_str(), child.cancellation().clone())) }
                })?
                .await?;
            assert!(std::ptr::eq(borrowed.as_ptr(), text.as_ptr()));
            assert!(closed.bail_if_cancelled().is_err());
            executor.cancellation().bail_if_cancelled()?;

            // The result is Send but not Sync. Caller policy may also borrow
            // non-Send state because it runs after the execution future.
            let observations = Rc::new(Cell::new(0));
            let observe = |value: Cell<usize>| {
                observations.set(observations.get() + 1);
                value.set(value.get() + observations.get());
                value.into_inner()
            };
            let value = executor
                .uncached_work(&AUTHENTICATION, |_| async { Ok(Cell::new(7)) })?
                .await?;
            assert_eq!(observe(value), 8);
            assert_eq!(observations.get(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn simultaneous_calls_execute_independently_and_close_each_child_scope() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let starts = AtomicUsize::new(0);
            let simultaneous = Barrier::new(2);
            let execute = || {
                executor.uncached_work(&AUTHENTICATION, |child| {
                    let starts = &starts;
                    let simultaneous = &simultaneous;
                    async move {
                        let number = starts.fetch_add(1, Ordering::SeqCst);
                        // Joining or serializing these calls would strand the
                        // first invocation at this barrier.
                        simultaneous.wait().await;
                        Ok((Cell::new(number), child.cancellation().clone()))
                    }
                })
            };
            let first = execute()?;
            let second = execute()?;
            assert_eq!(starts.load(Ordering::SeqCst), 0);
            let (first, second) =
                timeout(DEADLOCK_GUARD, async { tokio::join!(first, second) }).await?;
            let (first, first_scope) = first?;
            let (second, second_scope) = second?;
            assert_ne!(first.get(), second.get());
            assert_eq!(starts.load(Ordering::SeqCst), 2);
            assert!(first_scope.bail_if_cancelled().is_err());
            assert!(second_scope.bail_if_cancelled().is_err());
            executor.cancellation().bail_if_cancelled()?;
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancellation_prevents_unpolled_work_and_drops_an_inflight_attempt() -> Result<()> {
    for cancel_before_poll in [true, false] {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(AnonymousAuthenticationProvider),
                    scope,
                );
                let starts = AtomicUsize::new(0);
                let (started, started_scope) = oneshot::channel();
                let (release, released) = oneshot::channel();
                let mut operation = executor.uncached_work(&AUTHENTICATION, |child| {
                    let starts = &starts;
                    async move {
                        starts.fetch_add(1, Ordering::SeqCst);
                        started.send(child.cancellation().clone()).unwrap();
                        released.await?;
                        Ok(Cell::new(42))
                    }
                })?;
                let child_scope = if cancel_before_poll {
                    None
                } else {
                    assert_pending(operation.as_mut()).await;
                    Some(timeout(DEADLOCK_GUARD, started_scope).await??)
                };
                executor
                    .cancellation()
                    .request_cancel("cancel synthetic uncached work");
                assert!(timeout(DEADLOCK_GUARD, operation).await?.is_err());
                assert_eq!(
                    starts.load(Ordering::SeqCst),
                    usize::from(!cancel_before_poll)
                );
                // The execution owner must drop the callback, not leave a
                // detached task waiting for this release.
                assert!(release.send(()).is_err());
                if let Some(child_scope) = child_scope {
                    assert!(child_scope.bail_if_cancelled().is_err());
                }
                Ok::<_, eyre::Report>(())
            })
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn inherited_deferral_keeps_owned_work_alive_until_required_completion() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let (release, released) = oneshot::channel();
            let operation = executor.cancellable_scope(|child| async move {
                child
                    .deferred("finish synthetic owned result", |protected| async move {
                        protected
                            .uncached_work(&AUTHENTICATION, |work| async move {
                                released.await?;
                                Ok((Cell::new(42), work.cancellation().clone()))
                            })?
                            .await
                    })
                    .await
            });
            tokio::pin!(operation);
            assert_pending(operation.as_mut()).await;
            executor
                .cancellation()
                .request_cancel("finish protected synthetic work before shutdown");
            assert_pending(operation.as_mut()).await;
            release.send(()).unwrap();
            let (value, closed) = timeout(DEADLOCK_GUARD, operation).await??;
            assert_eq!(value.into_inner(), 42);
            assert!(closed.bail_if_cancelled().is_err());
            Ok(())
        })
        .await
}
