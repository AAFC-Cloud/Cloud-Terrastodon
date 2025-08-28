use crate::app_work::AcceptsWork;
use crate::app_work::AppWorkTracker;
use crate::app_work::AppboundWorkFinishedMessage;
use crate::app_work::UnboundedWorkFinishedMessageSender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

pub struct AppWorkState<State: 'static> {
    pub work_response_sender: UnboundedSender<AppboundWorkFinishedMessage<State>>,
    pub work_response_receiver: UnboundedReceiver<AppboundWorkFinishedMessage<State>>,
    pub work_tracker: AppWorkTracker<State>,
}
impl<State: 'static> Default for AppWorkState<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State: 'static> AppWorkState<State> {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppboundWorkFinishedMessage<State>>();
        Self {
            work_response_sender: tx,
            work_response_receiver: rx,
            work_tracker: AppWorkTracker::new(),
        }
    }
    pub fn handle_messages(&mut self, state: &mut State) -> eyre::Result<()> {
        while let Ok(msg) = self.work_response_receiver.try_recv() {
            match msg {
                AppboundWorkFinishedMessage::StateChange(mutator) => {
                    mutator.mutate_state(state);
                }
            }
        }
        self.work_tracker.prune()?;
        Ok(())
    }
}

impl<State: 'static> AcceptsWork<State> for AppWorkState<State> {
    fn work_tracker(&self) -> AppWorkTracker<State> {
        self.work_tracker.clone()
    }

    fn work_finished_message_sender(
        &self,
    ) -> impl crate::app_work::AppboundWorkFinishedMessageSender<State> {
        UnboundedWorkFinishedMessageSender::new(self.work_response_sender.clone())
    }
}
