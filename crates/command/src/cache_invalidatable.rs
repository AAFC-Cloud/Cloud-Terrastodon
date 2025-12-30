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

#[async_trait]
pub trait CacheInvalidatableIntoFuture: IntoFuture + CacheInvalidatable + Sized {
    async fn with_invalidation(self, invalidate_cache: bool) -> Self::Output;
}

#[async_trait]
impl<T, S> CacheInvalidatableIntoFuture for T
where
    T: IntoFuture<Output = eyre::Result<S>> + CacheInvalidatable + Sized + Send,
    T::IntoFuture: Send,
    S: Send,
{
    async fn with_invalidation(self, invalidate_cache: bool) -> Self::Output {
        if invalidate_cache {
            self.invalidate().await?;
        }
        self.into_future().await
    }
}
