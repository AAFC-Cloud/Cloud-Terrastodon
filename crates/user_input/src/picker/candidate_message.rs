use crate::Choice;

pub(super) struct CandidateMessage<T> {
    pub(super) generation: u64,
    pub(super) choices: Vec<Choice<T>>,
}
