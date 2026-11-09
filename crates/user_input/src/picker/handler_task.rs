use super::handler_completion::HandlerCompletion;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use teamy_cancellation::CancellationToken;

/// Owns a handler and the signal that can stop it without cancelling its peers.
pub(super) struct HandlerTask<'a> {
    pub(super) cancellation: CancellationToken,
    pub(super) future: Pin<Box<dyn Future<Output = HandlerCompletion> + Send + 'a>>,
}

impl Future for HandlerTask<'_> {
    type Output = HandlerCompletion;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().future.as_mut().poll(context)
    }
}
