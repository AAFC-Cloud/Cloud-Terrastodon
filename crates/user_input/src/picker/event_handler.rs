use super::candidate_sink::CandidateSink;
use super::handler_future::HandlerFuture;
use super::picker_event::PickerEvent;
use eyre::Report;
use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::Mutex;
use teamy_cancellation::CancellationScope;

pub(super) struct EventHandler<'a, T> {
    // Cloned registrations share a callback without requiring it to be Sync.
    // The lock protects only invocation; the returned future runs independently.
    #[expect(
        clippy::type_complexity,
        reason = "one stored picker callback signature"
    )]
    handler: Arc<
        Mutex<
            Box<
                dyn Fn(CancellationScope, Arc<PickerEvent>, CandidateSink<T>) -> HandlerFuture<'a>
                    + Send
                    + 'a,
            >,
        >,
    >,
}

impl<'a, T> EventHandler<'a, T> {
    pub(super) fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(CancellationScope, Arc<PickerEvent>, CandidateSink<T>) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<()>> + Send + 'a,
    {
        Self {
            handler: Arc::new(Mutex::new(Box::new(move |scope, event, sink| {
                Box::pin(handler(scope, event, sink))
            }))),
        }
    }

    pub(super) fn invoke(
        &self,
        scope: CancellationScope,
        event: Arc<PickerEvent>,
        sink: CandidateSink<T>,
    ) -> eyre::Result<HandlerFuture<'a>> {
        let handler = self
            .handler
            .lock()
            .map_err(|_| eyre::eyre!("picker event handler mutex poisoned"))?;
        // Catch while the guard is still owned outside the callback so a factory
        // panic cannot poison the registration for another generation.
        std::panic::catch_unwind(AssertUnwindSafe(|| handler(scope, event, sink)))
            .map_err(handler_panic_error)
    }
}

impl<T> Clone for EventHandler<'_, T> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
        }
    }
}

pub(super) fn handler_panic_error(payload: Box<dyn Any + Send>) -> Report {
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload");
    eyre::eyre!("picker event handler panicked: {message}")
}
