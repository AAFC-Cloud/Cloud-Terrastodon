use super::candidate_sink::CandidateSink;
use super::handler_future::HandlerFuture;
use super::picker_event::PickerEvent;
use std::sync::Arc;

pub(super) type EventHandler<'a, T> =
    Box<dyn Fn(Arc<PickerEvent>, CandidateSink<T>) -> HandlerFuture<'a> + Send + 'a>;
