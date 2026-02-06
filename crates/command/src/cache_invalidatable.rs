use crate::CacheKey;
use crate::HasCacheKey;
use async_trait::async_trait;

#[async_trait]
pub trait CacheInvalidatable {
    async fn invalidate(&self) -> eyre::Result<()>;
}

#[async_trait]
impl CacheInvalidatable for &CacheKey {
    async fn invalidate(&self) -> eyre::Result<()> {
        CacheKey::invalidate(self).await
    }
}

#[async_trait]
impl<T> CacheInvalidatable for T
where
    T: HasCacheKey + Sync,
{
    async fn invalidate(&self) -> eyre::Result<()> {
        let cache_key = self.cache_key();
        cache_key.invalidate().await
    }
}

pub trait CacheInvalidatableIntoFuture: IntoFuture + Sized {
    type WithInvalidation: IntoFuture<Output = Self::Output>;
    /// Invoke the command, optionally invalidating cache entries as part of execution.
    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation;
}
