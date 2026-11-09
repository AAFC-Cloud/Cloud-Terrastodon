use super::candidate_message::CandidateMessage;
use super::candidate_sink::CandidateSink;
use super::event_handler::EventHandler;
use super::event_handler::handler_panic_error;
use super::handler_completion::HandlerCompletion;
use super::handler_task::HandlerTask;
use super::picker_event::PickerEvent;
use futures::FutureExt;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;
use tokio::sync::mpsc;
use tracing::Instrument;
use tracing::trace_span;

/// Retains admitted picker work across reload and shutdown until it completes.
///
/// Current-generation counters describe visible loading work. Retired handlers
/// remain owned and polled, but cannot change those counters or fail the new UI.
pub(super) struct HandlerTasks<'a> {
    parent: CancellationScope,
    tasks: FuturesUnordered<HandlerTask<'a>>,
    generation: u64,
    pending: usize,
    startup: usize,
    accepting: bool,
}

impl<'a> HandlerTasks<'a> {
    pub(super) fn new(parent: CancellationScope) -> Self {
        Self {
            parent,
            tasks: FuturesUnordered::new(),
            generation: 0,
            pending: 0,
            startup: 0,
            accepting: true,
        }
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn pending(&self) -> usize {
        self.pending
    }

    pub(super) fn startup(&self) -> usize {
        self.startup
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub(super) fn spawn<T: Send + 'a>(
        &mut self,
        handlers: &[EventHandler<'a, T>],
        event: PickerEvent,
        is_startup: bool,
        sender: &mpsc::UnboundedSender<CandidateMessage<T>>,
    ) {
        if !self.accepting || self.parent.is_cancelled() {
            return;
        }
        let event_kind = match &event {
            PickerEvent::InitialLoad => "initial_load",
            PickerEvent::QueryChanged(_) => "query_changed",
            PickerEvent::QueryCleared => "query_cleared",
            PickerEvent::ReloadRequested(_) => "reload_requested",
        };
        let event = Arc::new(event);
        for handler in handlers {
            let handler = handler.clone();
            let event = Arc::clone(&event);
            let generation = self.generation;
            let sink = CandidateSink {
                sender: sender.clone(),
                generation,
            };
            let (cancellation, future) = self.parent.cancellable_child_scope(move |scope| {
                async move {
                    let future = handler.invoke(scope, event, sink)?;
                    AssertUnwindSafe(future)
                        .catch_unwind()
                        .await
                        .map_err(handler_panic_error)?
                }
                .instrument(trace_span!(
                    "picker_handler",
                    event = event_kind,
                    startup = is_startup,
                    generation,
                ))
            });
            self.tasks.push(HandlerTask {
                cancellation,
                future: Box::pin(async move {
                    HandlerCompletion {
                        generation,
                        is_startup,
                        result: future.await,
                    }
                }),
            });
            self.pending += 1;
            if is_startup {
                self.startup += 1;
            }
        }
    }

    pub(super) fn restart(&mut self) {
        for task in self.tasks.iter() {
            task.cancellation
                .request_cancel("picker generation was replaced");
        }
        self.generation = self.generation.wrapping_add(1);
        self.pending = 0;
        self.startup = 0;
    }

    pub(super) fn cancel(&mut self, reason: &str) {
        self.accepting = false;
        for task in self.tasks.iter() {
            task.cancellation.request_cancel(reason);
        }
    }

    pub(super) async fn next(&mut self) -> Option<eyre::Result<()>> {
        let completion = self.tasks.next().await?;
        if completion.generation == self.generation {
            self.pending = self.pending.saturating_sub(1);
            if completion.is_startup {
                self.startup = self.startup.saturating_sub(1);
            }
        }
        if let Err(error) = completion.result {
            if error.downcast_ref::<CancellationError>().is_some() {
                return Some(Ok(()));
            }
            if completion.generation != self.generation {
                tracing::warn!(
                    generation = completion.generation,
                    %error,
                    "retired picker handler failed during completion"
                );
                return Some(Ok(()));
            }
            self.cancel("a picker event handler failed");
            return Some(Err(error));
        }
        Some(Ok(()))
    }
}
