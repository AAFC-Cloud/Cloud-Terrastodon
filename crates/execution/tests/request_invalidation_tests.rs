#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

struct ObservationOperation;

struct CachedObservation<'a> {
    name: &'a str,
    source: &'a AtomicUsize,
    loads: &'a AtomicUsize,
    invalidation_keys: &'a AtomicUsize,
}

fn observation_key(name: &str) -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "request-invalidation", name])
            .unwrap(),
        valid_for: Duration::from_secs(3600),
    }
}

impl ExecutableRequest for CachedObservation<'_> {
    type Error = eyre::Report;
    type Output = Arc<usize>;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    observation_key(self.name),
                    CacheOperationOwner::of::<ObservationOperation>(),
                    &AuthenticationSelection::Anonymous,
                    NonEmptyString::new("synthetic-observation")?,
                    WorkReuse::ReusableRead,
                    move |_| async move {
                        self.loads.fetch_add(1, Ordering::SeqCst);
                        Ok(WorkOutput::new(
                            self.source.load(Ordering::SeqCst),
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await
        })
    }
}

impl HasCacheKey for CachedObservation<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        self.invalidation_keys.fetch_add(1, Ordering::SeqCst);
        Ok(observation_key(self.name))
    }
}

impl InvalidatesOwnCacheKey for CachedObservation<'_> {}

#[tokio::test]
async fn invalidation_is_lazy_and_refreshes_an_existing_cached_observation() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let source = AtomicUsize::new(7);
            let loads = AtomicUsize::new(0);
            let invalidation_keys = AtomicUsize::new(0);
            let request = || CachedObservation {
                name: "number",
                source: &source,
                loads: &loads,
                invalidation_keys: &invalidation_keys,
            };
            let original = executor.execute(request()).await?;
            source.store(9, Ordering::SeqCst);

            let unpolled = executor.execute(request()).with_invalidation(true);
            assert_eq!(invalidation_keys.load(Ordering::SeqCst), 0);
            assert_eq!(loads.load(Ordering::SeqCst), 1);
            drop(unpolled);

            let reused = request()
                .execute(&executor)
                .with_invalidation(false)
                .await?;
            assert!(Arc::ptr_eq(&original, &reused));
            assert_eq!(invalidation_keys.load(Ordering::SeqCst), 0);
            assert_eq!(loads.load(Ordering::SeqCst), 1);

            let refreshed = executor.execute(request()).with_invalidation(true).await?;
            assert_eq!(*refreshed, 9);
            assert_eq!(*original, 7);
            assert_eq!(invalidation_keys.load(Ordering::SeqCst), 1);
            assert_eq!(loads.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

struct ComposedObservation<'a> {
    first: CachedObservation<'a>,
    second: CachedObservation<'a>,
}

impl HasCacheKey for ComposedObservation<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(observation_key("composition"))
    }
}

impl CacheInvalidationKeys for ComposedObservation<'_> {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![self.first.cache_key()?, self.second.cache_key()?])
    }
}

impl ExecutableRequest for ComposedObservation<'_> {
    type Error = eyre::Report;
    type Output = (Arc<usize>, Arc<usize>);

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            let first = executor.execute(self.first).await?;
            let second = executor.execute(self.second).await?;
            Ok((first, second))
        })
    }
}

#[tokio::test]
async fn composed_invalidation_refreshes_both_children_and_preserves_unrelated_entries()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let sources = [
                AtomicUsize::new(10),
                AtomicUsize::new(20),
                AtomicUsize::new(30),
            ];
            let loads = AtomicUsize::new(0);
            let invalidation_keys = AtomicUsize::new(0);
            let request = |name: &'static str, index: usize| CachedObservation {
                name,
                source: &sources[index],
                loads: &loads,
                invalidation_keys: &invalidation_keys,
            };
            let first = executor.execute(request("first", 0)).await?;
            let second = executor.execute(request("second", 1)).await?;
            let unrelated = executor.execute(request("unrelated", 2)).await?;
            sources[0].store(11, Ordering::SeqCst);
            sources[1].store(21, Ordering::SeqCst);
            sources[2].store(31, Ordering::SeqCst);

            let refreshed = executor
                .execute(ComposedObservation {
                    first: request("first", 0),
                    second: request("second", 1),
                })
                .with_invalidation(true)
                .await?;
            assert_eq!((*refreshed.0, *refreshed.1), (11, 21));
            assert_eq!((*first, *second), (10, 20));
            let retained = executor.execute(request("unrelated", 2)).await?;
            assert!(Arc::ptr_eq(&unrelated, &retained));
            assert_eq!(*retained, 30);
            assert_eq!(loads.load(Ordering::SeqCst), 5);
            assert_eq!(invalidation_keys.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

struct RejectedInvalidation<'a> {
    runs: &'a AtomicUsize,
}

impl HasCacheKey for RejectedInvalidation<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        eyre::bail!("custom invalidation must not use the request's own cache key");
    }
}

impl ExecutableRequest for RejectedInvalidation<'_> {
    type Error = eyre::Report;
    type Output = ();

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            self.runs.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
}

impl CacheInvalidatableRequest for RejectedInvalidation<'_> {
    fn invalidate<'a>(
        &'a self,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(async {
            eyre::bail!("synthetic invalidation failed");
        })
    }
}

#[tokio::test]
async fn invalidation_failure_preserves_the_error_and_prevents_execution() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let runs = AtomicUsize::new(0);
            let error = executor
                .execute(RejectedInvalidation { runs: &runs })
                .with_invalidation(true)
                .await
                .unwrap_err();
            assert_eq!(error.to_string(), "synthetic invalidation failed");
            assert_eq!(runs.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

struct CancellingInvalidation<'a> {
    token: &'a CancellationToken,
    completed: &'a AtomicBool,
    runs: &'a AtomicUsize,
}

impl ExecutableRequest for CancellingInvalidation<'_> {
    type Error = eyre::Report;
    type Output = ();

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            self.runs.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
}

impl CacheInvalidatableRequest for CancellingInvalidation<'_> {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(executor.deferred(
            "finish synthetic invalidation",
            move |protected| async move {
                self.token
                    .request_cancel("cancel during synthetic invalidation");
                tokio::task::yield_now().await;
                protected.cancellation().bail_if_cancelled()?;
                self.completed.store(true, Ordering::SeqCst);
                Ok(())
            },
        ))
    }
}

#[tokio::test]
async fn cancellation_after_protected_invalidation_does_not_start_the_request() -> Result<()> {
    let token = CancellationToken::new();
    token
        .with_async_scope(|scope| async {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let completed = AtomicBool::new(false);
            let runs = AtomicUsize::new(0);
            let result = executor
                .execute(CancellingInvalidation {
                    token: &token,
                    completed: &completed,
                    runs: &runs,
                })
                .with_invalidation(true)
                .await;
            assert!(completed.load(Ordering::SeqCst));
            assert!(result.is_err());
            assert_eq!(runs.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

struct BorrowedCellRequest<'a> {
    label: &'a str,
    value: Cell<usize>,
}

impl<'input> ExecutableRequest for BorrowedCellRequest<'input> {
    type Error = eyre::Report;
    type Output = (&'input str, Cell<usize>, CancellationScope);

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move { Ok((self.label, self.value, executor.cancellation().clone())) })
    }
}

impl HasCacheKey for BorrowedCellRequest<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(observation_key("borrowed-cell"))
    }
}

impl InvalidatesOwnCacheKey for BorrowedCellRequest<'_> {}

#[tokio::test]
async fn both_spellings_support_blanket_invalidation_for_borrowed_non_sync_requests() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let label = String::from("borrowed offline input");
            let request = || BorrowedCellRequest {
                label: &label,
                value: Cell::new(7),
            };
            let first = executor.execute(request()).with_invalidation(true).await?;
            let second = request().execute(&executor).with_invalidation(true).await?;
            for (borrowed, value, child_scope) in [first, second] {
                assert!(std::ptr::eq(borrowed.as_ptr(), label.as_ptr()));
                value.set(9);
                assert_eq!(value.get(), 9);
                assert!(child_scope.bail_if_cancelled().is_err());
            }
            executor.cancellation().bail_if_cancelled()?;
            Ok(())
        })
        .await
}
