use super::StateMutator;

#[derive(Debug)]
pub enum AppboundWorkFinishedMessage<T> {
    StateChange(Box<dyn StateMutator<T>>),
}
impl<T> AppboundWorkFinishedMessage<T> {
    pub fn update_state(mutator: impl StateMutator<T>) -> Self {
        AppboundWorkFinishedMessage::StateChange(Box::new(mutator))
    }
}
