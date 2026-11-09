use super::Loadable;
use super::Setter;
use super::StateMutator;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::time::Instant;

/// Publishes a background failure to its application field.
pub struct FieldUpdaterWorkFailureMutator<State, FinishedWorkData> {
    pub err: eyre::Error,
    pub setter: Setter<State, FinishedWorkData>,
    pub started_at: Instant,
}

impl<State, FinishedWorkData> Debug for FieldUpdaterWorkFailureMutator<State, FinishedWorkData> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldUpdaterWorkFailureMutator")
            .finish_non_exhaustive()
    }
}

impl<State: 'static, FinishedWorkData> StateMutator<State>
    for FieldUpdaterWorkFailureMutator<State, FinishedWorkData>
where
    FinishedWorkData: Debug + Send + 'static,
{
    fn mutate_state(self: Box<Self>, state: &mut State) {
        (self.setter)(
            state,
            Loadable::Failed {
                error: self.err,
                started_at: self.started_at,
                finished_at: Instant::now(),
            },
        )
    }
}
