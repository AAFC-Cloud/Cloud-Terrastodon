use crate::CloudTerrastodonExecutor;
use crate::ExecutableRequest;
use crate::ExecutionFuture;
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

impl<'a, R> IntoFuture for RequestExecution<'a, R>
where
    R: ExecutableRequest + 'a,
    R::Output: 'a,
{
    type Output = Result<R::Output>;
    type IntoFuture = ExecutionFuture<'a, R::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(
            self.executor
                .cancellable_scope(move |child| self.request.run(child))
                .instrument(self.span),
        )
    }
}
