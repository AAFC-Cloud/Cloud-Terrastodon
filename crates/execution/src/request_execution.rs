use crate::CacheInvalidatableRequest;
use crate::CloudTerrastodonExecutor;
use crate::ExecutableRequest;
use crate::ExecutionFuture;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use eyre::Result;
use std::future::IntoFuture;
use tracing::Instrument;
use tracing::Span;

/// A request bound to its caller's executor and tracing context, ready to await.
#[must_use = "A bound request does nothing until awaited"]
pub struct RequestExecution<'a, R> {
    // Binding cannot be changed independently of the captured caller span.
    pub(crate) executor: &'a CloudTerrastodonExecutor,
    pub(crate) request: R,
    pub(crate) span: Span,
}

impl<'a, R> RequestExecution<'a, R>
where
    R: CacheInvalidatableRequest + 'a,
    R::Output: 'a,
{
    /// Invalidate this request through the same cache owner used for execution.
    pub fn invalidate(&self) -> ExecutionFuture<'_, ()> {
        self.request.invalidate(self.executor)
    }

    /// Optionally invalidate the request's cache entries before executing it.
    ///
    /// Both steps happen only when awaited, within the binding's tracing context
    /// and one linked child scope. Failed invalidation prevents execution;
    /// `false` executes without invoking the invalidation contract.
    /// Invalidation failures convert into the request's error without erasing
    /// errors returned by the request itself.
    pub fn with_invalidation(
        self,
        invalidate_cache: bool,
    ) -> ExecutionFuture<'a, R::Output, R::Error>
    where
        R::Error: From<eyre::Report> + 'a,
    {
        Box::pin(
            self.executor
                .cancellable_scope(move |child| async move {
                    if invalidate_cache {
                        self.request.invalidate(&child).await?;
                        // Invalidation can finish protected cleanup after cancellation.
                        // Starting the request is a separate work boundary.
                        child.cancellation().bail_if_cancelled()?;
                    }
                    self.request.run(child).await
                })
                .instrument(self.span),
        )
    }
}

impl<R: HasCacheKey> HasCacheKey for RequestExecution<'_, R> {
    fn cache_key(&self) -> Result<CacheKey> {
        self.request.cache_key()
    }
}

impl<'a, R> IntoFuture for RequestExecution<'a, R>
where
    R: ExecutableRequest + 'a,
    R::Output: 'a,
    R::Error: 'a,
{
    type Output = Result<R::Output, R::Error>;
    type IntoFuture = ExecutionFuture<'a, R::Output, R::Error>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(
            self.executor
                .cancellable_scope(move |child| self.request.run(child))
                .instrument(self.span),
        )
    }
}
