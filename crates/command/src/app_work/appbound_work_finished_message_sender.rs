use super::AppboundWorkFinishedMessage;

pub trait AppboundWorkFinishedMessageSender<State>: Sized + Send + 'static {
    fn send(&self, msg: AppboundWorkFinishedMessage<State>) -> eyre::Result<()>;
}
