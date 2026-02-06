use crate::CacheKey;
use crate::HasCacheKey;
use async_trait::async_trait;

/// Convenience trait for implementing commands that can be cached.
/// This command gives [`HasCacheKey`] and [`InvalidatableCommand`] implementations.
#[async_trait]
pub trait CacheableCommand: Sized + Send {
    type Output;

    fn cache_key(&self) -> CacheKey;
    async fn run(self) -> eyre::Result<Self::Output>;
}

impl<T> HasCacheKey for T
where
    T: CacheableCommand,
{
    fn cache_key(&self) -> CacheKey {
        CacheableCommand::cache_key(self)
    }
}

/// Implement `IntoFuture` and `CacheInvalidatableIntoFuture` for a concrete
/// `CacheableCommand` type without repeating boilerplate.
///
/// Usage:
/// - `impl_cacheable_into_future!(MyCommandType);`
/// - `impl_cacheable_into_future!(MyCommandType<'a>, 'a);` for request types that carry a lifetime
///   and therefore need the boxed future to be bounded by that lifetime.
#[macro_export]
macro_rules! impl_cacheable_into_future {
    // No lifetime: produces a boxed future without an explicit lifetime bound.
    ($ty:ty) => {
        impl ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableCommand>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::CacheableCommand::run(self))
            }
        }

        impl $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output> + Send>>;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <Self as $crate::CacheInvalidatable>::invalidate(&self).await?;
                    }
                    $crate::CacheableCommand::run(self).await
                })
            }
        }
    };

    // With an explicit lifetime: attaches the lifetime bound to the boxed future.
    ($ty:ty, $lt:lifetime) => {
        impl<$lt> ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableCommand>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send + $lt>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::CacheableCommand::run(self))
            }
        }

        impl<$lt> $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output> + Send + $lt>>;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <Self as $crate::CacheInvalidatable>::invalidate(&self).await?;
                    }
                    $crate::CacheableCommand::run(self).await
                })
            }
        }
    };

    // With explicit type generics (e.g., `T: Scope + Send`): adds the generics to the impl
    // but does not attempt to add a lifetime bound to the boxed future.
    ($ty:ty, $($gens:tt)+) => {
        impl<$($gens)+> ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableCommand>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::CacheableCommand::run(self))
            }
        }

        impl<$($gens)+ + Sync> $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output> + Send>>;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <Self as $crate::CacheInvalidatable>::invalidate(&self).await?;
                    }
                    $crate::CacheableCommand::run(self).await
                })
            }
        }
    };

    // With both a lifetime and generics: e.g., `MyReq<'a, T>, 'a, T: Foo`
    ($ty:ty, $lt:lifetime, $($gens:tt)+) => {
        impl<$lt, $($gens)+> ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableCommand>::Output>;
            type IntoFuture = ::std::pin::Pin<
                Box<dyn ::std::future::Future<Output = Self::Output> + Send + $lt>
            >;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::CacheableCommand::run(self))
            }
        }

        impl<$lt, $($gens)+ + Sync> $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation = ::std::pin::Pin<
                Box<dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output> + Send + $lt>
            >;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <Self as $crate::CacheInvalidatable>::invalidate(&self).await?;
                    }
                    $crate::CacheableCommand::run(self).await
                })
            }
        }
    };
}
