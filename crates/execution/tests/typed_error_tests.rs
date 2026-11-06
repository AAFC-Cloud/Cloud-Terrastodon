#![cfg(test)]

use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationToken;

// Deliberately neither Sync nor convertible into an eyre report. A direct
// invocation can retain borrowed recovery data without a registry contract.
#[derive(Debug)]
enum RequestFailure<'a> {
    Partial(Cell<&'a str>),
    Cancelled(CancellationError),
    Invalidation(eyre::Report),
}

impl From<CancellationError> for RequestFailure<'_> {
    fn from(error: CancellationError) -> Self {
        Self::Cancelled(error)
    }
}

impl From<eyre::Report> for RequestFailure<'_> {
    fn from(error: eyre::Report) -> Self {
        Self::Invalidation(error)
    }
}

struct FailingRequest<'a> {
    choice: &'a str,
    runs: &'a AtomicUsize,
    invalidations: &'a AtomicUsize,
    reject_invalidation: bool,
}

impl<'input> ExecutableRequest for FailingRequest<'input> {
    type Output = ();
    type Error = RequestFailure<'input>;

    fn run<'a>(
        self,
        _executor: CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Self::Output, Self::Error>
    where
        Self: 'a,
        Self::Output: 'a,
        Self::Error: 'a,
    {
        Box::pin(async move {
            self.runs.fetch_add(1, Ordering::SeqCst);
            Err(RequestFailure::Partial(Cell::new(self.choice)))
        })
    }
}

impl CacheInvalidatableRequest for FailingRequest<'_> {
    fn invalidate<'a>(
        &'a self,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            self.invalidations.fetch_add(1, Ordering::SeqCst);
            if self.reject_invalidation {
                return Err(std::io::Error::other("synthetic invalidation failure").into());
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn both_execution_spellings_preserve_borrowed_non_sync_recovery_data() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let choice = String::from("selected item");
            let runs = AtomicUsize::new(0);
            let invalidations = AtomicUsize::new(0);
            let request = || FailingRequest {
                choice: &choice,
                runs: &runs,
                invalidations: &invalidations,
                reject_invalidation: false,
            };
            let failures = [
                executor.execute(request()).await.unwrap_err(),
                request().execute(&executor).await.unwrap_err(),
            ];
            for failure in failures {
                let RequestFailure::Partial(selected) = failure else {
                    panic!("the request's recovery data was lost: {failure:?}");
                };
                assert!(std::ptr::eq(selected.get().as_ptr(), choice.as_ptr()));
                selected.set("recovered item");
                assert_eq!(selected.get(), "recovered item");
            }
            assert_eq!(runs.load(Ordering::SeqCst), 2);
            assert_eq!(invalidations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn invalidation_preserves_request_errors_and_converts_its_own_failure() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let choice = String::from("selected item");
            let runs = AtomicUsize::new(0);
            let invalidations = AtomicUsize::new(0);
            let request = |reject_invalidation| FailingRequest {
                choice: &choice,
                runs: &runs,
                invalidations: &invalidations,
                reject_invalidation,
            };
            for invalidate in [false, true] {
                let failure = request(false)
                    .execute(&executor)
                    .with_invalidation(invalidate)
                    .await
                    .unwrap_err();
                let RequestFailure::Partial(selected) = failure else {
                    panic!("invalidation erased the request error: {failure:?}");
                };
                assert!(std::ptr::eq(selected.get().as_ptr(), choice.as_ptr()));
                assert_eq!(
                    invalidations.load(Ordering::SeqCst),
                    usize::from(invalidate)
                );
            }
            let failure = executor
                .execute(request(true))
                .with_invalidation(true)
                .await
                .unwrap_err();
            let RequestFailure::Invalidation(error) = failure else {
                panic!("invalidation failure used the wrong error path: {failure:?}");
            };
            assert_eq!(
                error.downcast_ref::<std::io::Error>().unwrap().to_string(),
                "synthetic invalidation failure"
            );
            assert_eq!(invalidations.load(Ordering::SeqCst), 2);
            assert_eq!(runs.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelled_execution_returns_a_typed_cause_without_starting_the_request() -> Result<()> {
    let token = CancellationToken::new();
    token
        .with_async_scope(|scope| async {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let runs = AtomicUsize::new(0);
            let invalidations = AtomicUsize::new(0);
            token.request_cancel("synthetic caller cancellation");
            let failure = executor
                .execute(FailingRequest {
                    choice: "selected item",
                    runs: &runs,
                    invalidations: &invalidations,
                    reject_invalidation: false,
                })
                .await
                .unwrap_err();
            let RequestFailure::Cancelled(error) = failure else {
                panic!("execution lost the typed cancellation cause: {failure:?}");
            };
            assert!(error.to_string().contains("synthetic caller cancellation"));
            assert_eq!(runs.load(Ordering::SeqCst), 0);
            assert_eq!(invalidations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}
