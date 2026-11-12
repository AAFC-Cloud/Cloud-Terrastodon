use crate::CacheInvalidatableRequest;
use crate::CloudTerrastodonExecutor;
use crate::ExecutableRequest;
use crate::ExecutionEvent;
use crate::ExecutionFuture;
use crate::ObservedRequestExecution;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use eyre::Result;
use std::future::IntoFuture;
use tracing::Instrument;
use tracing::Span;

/// A request bound to its caller's executor and tracing context, ready to await.
#[must_use = "A bound request does nothing until awaited"]
pub struct RequestExecution<'a, R: ExecutableRequest> {
    // Binding cannot be changed independently of the captured caller span.
    pub(crate) executor: &'a CloudTerrastodonExecutor,
    pub(crate) request: R,
    pub(crate) span: Span,
    #[allow(
        clippy::type_complexity,
        reason = "Retain optional invalidation's trait witness without requiring every executable request to support it"
    )]
    pub(crate) invalidation: Option<
        for<'b> fn(&'b R, &'b CloudTerrastodonExecutor) -> ExecutionFuture<'b, (), R::Error>,
    >,
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
    pub fn with_invalidation(mut self, invalidate_cache: bool) -> Self
    where
        R::Error: From<eyre::Report> + 'a,
    {
        self.invalidation = invalidate_cache.then_some(|request, executor| {
            let invalidation = request.invalidate(executor);
            Box::pin(async move { invalidation.await.map_err(R::Error::from) })
        });
        self
    }
}

impl<R: ExecutableRequest + HasCacheKey> HasCacheKey for RequestExecution<'_, R> {
    fn cache_key(&self) -> Result<CacheKey> {
        self.request.cache_key()
    }
}

impl<'a, R> RequestExecution<'a, R>
where
    R: ExecutableRequest + 'a,
    R::Output: 'a,
    R::Error: 'a,
{
    /// Observe this caller's lifecycle and initializer progress in the caller's
    /// task. The callback may borrow local state and need not be Send or Sync.
    /// Notifications must return promptly without synchronously waiting for
    /// execution or cache work. Dropping the bound future emits no terminal event.
    pub fn observe<F>(self, observer: F) -> ObservedRequestExecution<'a, R::Output, R::Error, F>
    where
        F: FnMut(ExecutionEvent),
    {
        let (sender, events) = tokio::sync::mpsc::unbounded_channel();
        let executor = self.executor.with_observer(sender);
        ObservedRequestExecution {
            work: self.run(executor, true),
            events,
            observer: Box::new(observer),
        }
    }

    fn run(
        self,
        executor: CloudTerrastodonExecutor,
        observe_admission: bool,
    ) -> ExecutionFuture<'a, R::Output, R::Error> {
        Box::pin(
            executor
                .cancellable_scope(move |child| async move {
                    let child = self.request.bind_authentication(child)?;
                    child
                        .with_account_cancellation(move |child| async move {
                            if observe_admission {
                                child.report_event(ExecutionEvent::Admitted);
                            }
                            if let Some(invalidate) = self.invalidation {
                                invalidate(&self.request, &child).await?;
                                child.cancellation().bail_if_cancelled()?;
                            }
                            self.request.run(child).await
                        })
                        .await
                })
                .instrument(self.span),
        )
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
        let executor = self.executor.clone();
        self.run(executor, false)
    }
}
