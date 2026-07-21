use super::choice_pool::ChoicePool;
use compact_str::CompactString;
use rustc_hash::FxHashSet;

/// Headless state shared by the event loop's candidate and reload semantics.
/// The terminal and Nucleo remain owned by the UI task; this type only models
/// the state transitions that need to remain deterministic as handlers finish.
#[derive(Debug)]
pub(super) struct PickerEventState<T> {
    pub(super) candidates: ChoicePool<T>,
    pub(super) marked: FxHashSet<CompactString>,
    pub(super) generation: u64,
}

impl<T> Default for PickerEventState<T> {
    fn default() -> Self {
        Self {
            candidates: ChoicePool::default(),
            marked: FxHashSet::default(),
            generation: 0,
        }
    }
}

impl<T> PickerEventState<T> {
    pub(super) fn reload(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.candidates.clear();
        self.marked.clear();
    }
}
