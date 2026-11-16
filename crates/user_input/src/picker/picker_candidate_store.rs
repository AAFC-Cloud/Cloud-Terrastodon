use super::candidate_message::CandidateMessage;
use super::event_handler::EventHandler;
use super::handler_tasks::HandlerTasks;
use super::picker_candidates::PickerCandidates;
use super::picker_event::PickerEvent;
use super::picker_event_state::PickerEventState;
use super::should_warn_for_tab::should_warn_for_tab;
use crate::PickError;
use crate::PickManyResult;
use compact_str::CompactString;
use nucleo::Nucleo;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;

/// Own the typed candidate values and handlers throughout one picker session.
///
/// No values are erased or cloned: the UI borrows the key-only interface, then
/// selected values move directly from this store into the caller's result.
pub(super) struct PickerCandidateStore<'a, T> {
    state: PickerEventState<T>,
    handlers: Vec<EventHandler<'a, T>>,
    sender: mpsc::UnboundedSender<CandidateMessage<T>>,
    receiver: mpsc::UnboundedReceiver<CandidateMessage<T>>,
}

impl<'a, T> PickerCandidateStore<'a, T> {
    pub(super) fn new(
        handlers: Vec<EventHandler<'a, T>>,
        marked: FxHashSet<CompactString>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            state: PickerEventState {
                marked,
                ..PickerEventState::default()
            },
            handlers,
            sender,
            receiver,
        }
    }

    pub(super) fn finish(self, outcome: Result<(), PickError<()>>) -> PickManyResult<T> {
        match outcome {
            Ok(()) => Ok(self.state.selected_values()),
            Err(PickError::Eyre(error, _)) => {
                Err(PickError::Eyre(error, self.state.selected_values()))
            }
            Err(PickError::Cancelled) => Err(PickError::Cancelled),
            Err(PickError::NoChoicesProvided) => Err(PickError::NoChoicesProvided),
            Err(PickError::ReloadRequested) => Err(PickError::ReloadRequested),
        }
    }
}

impl<'a, T: Send + 'a> PickerCandidates<'a> for PickerCandidateStore<'a, T> {
    fn spawn_handlers(&self, tasks: &mut HandlerTasks<'a>, event: PickerEvent, startup: bool) {
        tasks.spawn(&self.handlers, event, startup, &self.sender);
    }

    fn poll_update(
        &mut self,
        cx: &mut Context<'_>,
        nucleo: &mut Nucleo<Arc<CompactString>>,
        warned_tab_keys: &mut FxHashSet<CompactString>,
    ) -> Poll<Option<(bool, Vec<CompactString>)>> {
        self.receiver.poll_recv(cx).map(|message| {
            message.map(|message| {
                process_candidate_message(
                    message,
                    self.state.generation,
                    &mut self.state,
                    nucleo,
                    warned_tab_keys,
                )
            })
        })
    }

    fn try_update(
        &mut self,
        nucleo: &mut Nucleo<Arc<CompactString>>,
        warned_tab_keys: &mut FxHashSet<CompactString>,
    ) -> Option<(bool, Vec<CompactString>)> {
        self.receiver.try_recv().ok().map(|message| {
            process_candidate_message(
                message,
                self.state.generation,
                &mut self.state,
                nucleo,
                warned_tab_keys,
            )
        })
    }

    fn generation(&self) -> u64 {
        self.state.generation
    }

    fn len(&self) -> usize {
        self.state.candidates.len()
    }

    fn channel_is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.state.candidates.contains_key(key)
    }

    fn marked(&self) -> &FxHashSet<CompactString> {
        &self.state.marked
    }

    fn marked_mut(&mut self) -> &mut FxHashSet<CompactString> {
        &mut self.state.marked
    }

    fn reload(&mut self) {
        self.state.reload();
    }
}

#[cfg(feature = "extended_observability")]
macro_rules! extended_trace_span {
    ($name:literal) => {
        tracing::trace_span!($name)
    };
}

#[cfg(not(feature = "extended_observability"))]
macro_rules! extended_trace_span {
    ($name:literal) => {
        tracing::Span::none()
    };
}

pub(super) fn process_candidate_message<T>(
    message: CandidateMessage<T>,
    generation: u64,
    picker_state: &mut PickerEventState<T>,
    nucleo: &mut Nucleo<Arc<CompactString>>,
    warned_tab_keys: &mut FxHashSet<CompactString>,
) -> (bool, Vec<CompactString>) {
    if message.generation != generation {
        return (false, Vec::new());
    }

    let mut tab_warning_keys = Vec::new();
    let changed = extended_trace_span!("picker_inject_candidates").in_scope(|| {
        picker_state.candidates.inject(message.choices, |key| {
            nucleo.injector().push(Arc::new(key.clone()), |x, cols| {
                cols[0] = x.as_ref().as_str().into();
            });
            if should_warn_for_tab(warned_tab_keys, key) {
                tab_warning_keys.push(key.clone());
            }
        })
    });
    (changed, tab_warning_keys)
}

#[cfg(test)]
mod tests {
    use super::PickerCandidateStore;
    use crate::Choice;
    use crate::PickError;
    use crate::picker::candidate_message::CandidateMessage;
    use crate::picker::picker_candidates::PickerCandidates;
    use crate::picker::picker_ui::new_nucleo;
    use compact_str::CompactString;
    use rustc_hash::FxHashSet;
    use std::future::poll_fn;

    #[tokio::test]
    async fn erased_ui_returns_borrowed_non_clone_choices_on_success_and_failure() {
        for failure in [false, true] {
            let mut value = String::from("borrowed");
            let mut store = PickerCandidateStore::new(Vec::new(), FxHashSet::default());
            store
                .sender
                .send(CandidateMessage {
                    generation: 0,
                    choices: vec![Choice {
                        key: "selected".to_owned(),
                        value: &mut value,
                    }],
                })
                .unwrap();
            let mut nucleo = new_nucleo();
            let mut warned = FxHashSet::default();
            let candidates: &mut dyn PickerCandidates<'_> = &mut store;
            let update = poll_fn(|cx| candidates.poll_update(cx, &mut nucleo, &mut warned)).await;
            assert_eq!(update, Some((true, Vec::new())));
            candidates
                .marked_mut()
                .insert(CompactString::from("selected"));

            let outcome = if failure {
                Err(eyre::eyre!("handler failed").into())
            } else {
                Ok(())
            };
            let selected = match store.finish(outcome) {
                Ok(selected) => {
                    assert!(!failure);
                    selected
                }
                Err(PickError::Eyre(error, selected)) => {
                    assert!(failure);
                    assert_eq!(error.to_string(), "handler failed");
                    selected
                }
                Err(error) => panic!("unexpected picker outcome: {error}"),
            };
            assert_eq!(selected.len(), 1);
            for selected in selected {
                selected.push_str(" value");
            }
            assert_eq!(value, "borrowed value");
        }
    }
}
