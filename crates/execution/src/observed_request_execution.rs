use crate::ExecutionEvent;
use crate::ExecutionFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc::UnboundedReceiver;

/// Delivers one invocation's events in the caller's task while polling its work.
///
/// A borrowed or non-Send callback is supported without making the underlying
/// request non-Send or spawning detached work. Send callbacks keep this future Send.
/// Callbacks are notifications: they must return promptly and must not
/// synchronously wait for execution or cache work. The suspended request can
/// retain initializer or artifact I/O ownership while notifications are delivered.
/// Dropping this future drops its callback and work without a terminal event.
#[must_use = "Observed execution does nothing until awaited"]
pub struct ObservedRequestExecution<'a, T, E, F> {
    pub(crate) work: ExecutionFuture<'a, T, E>,
    pub(crate) events: UnboundedReceiver<ExecutionEvent>,
    pub(crate) observer: Box<F>,
}

impl<T, E, F> Future for ObservedRequestExecution<'_, T, E, F>
where
    F: FnMut(ExecutionEvent),
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // User code runs after the producer's poll stack has unwound, outside
        // synchronous cache state critical sections. This is not an execution
        // suspension point that permits synchronously awaiting that same work.
        let result = this.work.as_mut().poll(cx);
        match &result {
            Poll::Ready(result) => {
                // poll_recv may yield for Tokio's cooperative budget even while
                // events remain queued. Once work is done, drain its finite
                // queue before returning the result and dropping the receiver.
                while let Ok(event) = this.events.try_recv() {
                    (this.observer)(event);
                }
                (this.observer)(if result.is_ok() {
                    ExecutionEvent::Completed
                } else {
                    ExecutionEvent::Failed
                });
            }
            Poll::Pending => {
                // Register event wakeups while work is pending, retaining Tokio's
                // cooperative yielding for a long stream of notifications.
                while let Poll::Ready(Some(event)) = this.events.poll_recv(cx) {
                    (this.observer)(event);
                }
            }
        }
        result
    }
}
