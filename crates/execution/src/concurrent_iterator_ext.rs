use crate::CloudTerrastodonExecutor;
use crate::map_concurrent::map_concurrent;
use std::future::Future;
use std::future::IntoFuture;
use std::num::NonZeroUsize;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;

/// Bounded asynchronous collection of a finite iterator under linked child scopes.
///
/// These are terminal operations returning futures, rather than lazy iterator or
/// stream adapters. Work starts when polled and results retain input order. Each
/// factory receives its own child scope or executor and must use it for the work.
/// Futures run inline and may borrow inputs; no task spawning or additional
/// `Clone`/`Send`/`'static` requirements are imposed.
///
/// The first observed failure stops admission, cancels the group, and drains
/// active children before returning the original error. Parent cancellation
/// propagates through linked scopes; inherited deferrals remain effective.
/// To attempt every item despite ordinary operation failures, return
/// `Ok(operation_result)` and inspect the inner results after the map drains.
/// Batch cancellation returns an error instead of partial results, including
/// collected operation errors.
///
/// Callers must retain the returned future until it finishes. Explicit drops,
/// task aborts, panics, and enclosing fail-fast combinators can abandon the group.
pub trait ConcurrentIteratorExt: Iterator + Sized {
    /// Collect concurrent operations using child executors that preserve shared
    /// resources, authentication restrictions, observation, and execution ancestry.
    ///
    /// Use [`Self::map_concurrent_scoped`] for work needing only cancellation.
    #[track_caller]
    fn map_concurrent<C, F, T, E>(
        self,
        executor: &CloudTerrastodonExecutor,
        limit: NonZeroUsize,
        mut factory: C,
    ) -> impl Future<Output = Result<Vec<T>, E>> + use<Self, C, F, T, E>
    where
        C: FnMut(CloudTerrastodonExecutor, Self::Item) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = executor.clone();
        self.map_concurrent_scoped(executor.cancellation(), limit, move |scope, item| {
            factory(parent.with_scope(scope), item)
        })
    }

    /// Collect concurrent operations using child cancellation scopes, without
    /// requiring an executor. See the trait's cancellation and draining contract.
    ///
    /// ```
    /// use cloud_terrastodon_execution::ConcurrentIteratorExt;
    /// use std::num::NonZeroUsize;
    /// use teamy_cancellation::{CancellationError, CancellationToken};
    ///
    /// # async fn example() -> Result<(), CancellationError> {
    /// let lengths = CancellationToken::new()
    ///     .with_async_scope(|scope| async move {
    ///         ["one", "three"]
    ///             .iter()
    ///             .map_concurrent_scoped(&scope, NonZeroUsize::MIN, |_child, text| async move {
    ///                 Ok::<_, CancellationError>(text.len())
    ///             })
    ///             .await
    ///     })
    ///     .await?;
    /// assert_eq!(lengths, [3, 5]);
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    fn map_concurrent_scoped<C, F, T, E>(
        self,
        cancellation: &CancellationScope,
        limit: NonZeroUsize,
        factory: C,
    ) -> impl Future<Output = Result<Vec<T>, E>> + use<Self, C, F, T, E>
    where
        C: FnMut(CancellationScope, Self::Item) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        map_concurrent(cancellation, self, limit, factory)
    }
}

impl<I: Iterator> ConcurrentIteratorExt for I {}
