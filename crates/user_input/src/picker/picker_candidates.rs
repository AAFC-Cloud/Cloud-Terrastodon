use super::handler_tasks::HandlerTasks;
use super::picker_event::PickerEvent;
use compact_str::CompactString;
use nucleo::Nucleo;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

/// Candidate ownership and event work seen by the value-independent terminal UI.
///
/// Values stay in the typed store. The UI handles only their display keys, so its
/// event loop does not need a separate machine-code instance for every value type.
pub(super) trait PickerCandidates<'a>: Send {
    fn spawn_handlers(&self, tasks: &mut HandlerTasks<'a>, event: PickerEvent, startup: bool);

    fn poll_update(
        &mut self,
        cx: &mut Context<'_>,
        nucleo: &mut Nucleo<Arc<CompactString>>,
        warned_tab_keys: &mut FxHashSet<CompactString>,
    ) -> Poll<Option<(bool, Vec<CompactString>)>>;

    fn try_update(
        &mut self,
        nucleo: &mut Nucleo<Arc<CompactString>>,
        warned_tab_keys: &mut FxHashSet<CompactString>,
    ) -> Option<(bool, Vec<CompactString>)>;

    fn generation(&self) -> u64;
    fn len(&self) -> usize;
    fn channel_is_empty(&self) -> bool;
    fn contains_key(&self, key: &str) -> bool;
    fn marked(&self) -> &FxHashSet<CompactString>;
    fn marked_mut(&mut self) -> &mut FxHashSet<CompactString>;
    fn reload(&mut self);
}
