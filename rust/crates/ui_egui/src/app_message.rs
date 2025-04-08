use crate::state_mutator::StateMutator;

#[derive(Debug)]
pub enum AppMessage {
    StateChange(Box<dyn StateMutator>),
}
impl AppMessage {
    pub fn update_state(mutator: impl StateMutator + 'static) -> Self {
        AppMessage::StateChange(Box::new(mutator))
    }
}
