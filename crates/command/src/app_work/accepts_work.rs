use super::AppWorkTracker;
use super::AppboundWorkFinishedMessageSender;
use tokio::runtime::Handle;

pub trait AcceptsWork<State>: Sized {
    fn work_tracker(&self) -> AppWorkTracker<State>;
    fn work_finished_message_sender(&self) -> impl AppboundWorkFinishedMessageSender<State>;
    fn runtime(&self) -> Handle {
        Handle::current()
    }
}
