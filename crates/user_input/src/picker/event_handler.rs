use super::candidate_sink::CandidateSink;
use super::handler_future::HandlerFuture;
use super::picker_event::PickerEvent;

pub(super) struct EventHandler<'a, T> {
    pub(super) handler:
        Box<dyn Fn(std::sync::Arc<PickerEvent>, CandidateSink<T>) -> HandlerFuture<'a> + Send + 'a>,
}
