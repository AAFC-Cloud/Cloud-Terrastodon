use crate::CacheKey;
use crate::HasCacheKey;
use async_trait::async_trait;

#[async_trait]
pub trait CacheableCommand: Sized + 'static + Send {
    type Output;

    fn cache_key(&self) -> CacheKey;
    async fn run(self) -> eyre::Result<Self::Output>;
    async fn run_with_invalidation(self, invalidate_cache: bool) -> eyre::Result<Self::Output> {
        if invalidate_cache {
            self.cache_key().invalidate().await?;
        }
        self.run().await
    }
}

/// Implement `IntoFuture` for a concrete `CacheableCommand` type without repeating boilerplate.
///
/// Usage: `impl_cacheable_into_future!(MyCommandType);`
#[macro_export]
macro_rules! impl_cacheable_into_future {
    ($ty:ty) => {
        impl ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableCommand>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::CacheableCommand::run(self))
            }
        }
    };
}

impl<T> HasCacheKey for T
where
    T: CacheableCommand,
{
    fn cache_key(&self) -> CacheKey {
        CacheableCommand::cache_key(self)
    }
}
