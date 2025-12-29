use crate::HasCacheKey;
use async_trait::async_trait;
use std::borrow::Cow;
use std::path::PathBuf;

#[async_trait]
pub trait CacheableCommand: Sized + 'static + Send {
    type Output;

    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf>;
    async fn run(self) -> eyre::Result<Self::Output>;
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
    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf> {
        CacheableCommand::cache_key(self)
    }
}
