use super::candidate_sink::CandidateSink;
use super::handler_future::HandlerFuture;
use super::picker_event::PickerEvent;
use std::sync::Arc;

pub(super) struct EventHandler<'a, T> {
    pub(super) label: Arc<str>,
    pub(super) handler:
        Box<dyn Fn(Arc<PickerEvent>, CandidateSink<T>) -> HandlerFuture<'a> + Send + 'a>,
}
