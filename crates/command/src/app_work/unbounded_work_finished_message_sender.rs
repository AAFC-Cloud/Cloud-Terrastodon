use super::AppboundWorkFinishedMessage;
use super::AppboundWorkFinishedMessageSender;
use eyre::eyre;
use tokio::sync::mpsc::UnboundedSender;

pub struct UnboundedWorkFinishedMessageSender<State> {
    pub sender: UnboundedSender<AppboundWorkFinishedMessage<State>>,
}

impl<State> UnboundedWorkFinishedMessageSender<State> {
    pub fn new(sender: UnboundedSender<AppboundWorkFinishedMessage<State>>) -> Self {
        Self { sender }
    }
}

impl<State: 'static> AppboundWorkFinishedMessageSender<State>
    for UnboundedWorkFinishedMessageSender<State>
{
    fn send(&self, msg: AppboundWorkFinishedMessage<State>) -> eyre::Result<()> {
        self.sender.send(msg).map_err(|error| {
            eyre!("Cannot deliver background work because its UI receiver closed: {error}")
        })
    }
}
