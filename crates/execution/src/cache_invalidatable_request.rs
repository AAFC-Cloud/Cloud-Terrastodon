use crate::CloudTerrastodonExecutor;
use crate::ExecutableRequest;
use crate::ExecutionFuture;
use cloud_terrastodon_cache::CacheInvalidationKeys;

/// An executable request that owns a cache-invalidation contract.
///
/// Requests implementing [`CacheInvalidationKeys`] receive shared invalidation
/// of their selected scopes and descendants. Opt into
/// [`cloud_terrastodon_cache::InvalidatesOwnCacheKey`] for the single-key default.
/// For custom asynchronous invalidation, implement this trait directly instead
/// of either key-selection policy. Each choice can be used by a request that
/// also implements [`cloud_terrastodon_cache::HasCacheKey`].
/// Binding with [`crate::RequestExecution::with_invalidation`] keeps this work
/// lazy and uses the same child executor for invalidation and execution.
pub trait CacheInvalidatableRequest: ExecutableRequest {
    /// Invalidate this request's cache entries using the caller's resources.
    ///
    /// Preserve the supplied cancellation scope and propagate failures so a
    /// refresh cannot silently proceed with entries that failed to invalidate.
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>;
}

impl<T> CacheInvalidatableRequest for T
where
    T: ExecutableRequest + CacheInvalidationKeys,
{
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        // Own all keys before awaiting so request data need only be Send, not Sync.
        let keys = self.cache_keys_to_invalidate();
        Box::pin(async move {
            let keys = keys?;
            let cache = executor.cache()?;
            // This is a sequence, not an atomic batch. On failure, earlier
            // invalidations remain effective and execution must not begin.
            for key in keys {
                cache
                    .invalidate(&key.scope, executor.cancellation())
                    .await?;
            }
            Ok(())
        })
    }
}
