use super::Loadable;
use super::Setter;
use super::StateMutator;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::time::Instant;

/// Publishes a completed background result to its application field.
pub struct FieldUpdaterWorkSuccessMutator<State, FinishedWorkData> {
    pub data: FinishedWorkData,
    pub setter: Setter<State, FinishedWorkData>,
    pub started_at: Instant,
}

impl<State, FinishedWorkData> Debug for FieldUpdaterWorkSuccessMutator<State, FinishedWorkData> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldUpdaterWorkSuccessMutator")
            .finish_non_exhaustive()
    }
}

impl<State: 'static, FinishedWorkData> StateMutator<State>
    for FieldUpdaterWorkSuccessMutator<State, FinishedWorkData>
where
    FinishedWorkData: Debug + Send + 'static,
{
    fn mutate_state(self: Box<Self>, state: &mut State) {
        (self.setter)(
            state,
            Loadable::Loaded {
                started_at: self.started_at,
                finished_at: Instant::now(),
                value: self.data,
            },
        )
    }
}
