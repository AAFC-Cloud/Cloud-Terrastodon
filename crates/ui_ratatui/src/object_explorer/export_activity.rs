use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Keeps the UI aware of an export from before admission until its lease closes.
/// Ownership survives moving the export future between tasks and releases on abort.
#[derive(Debug)]
pub(crate) struct ExportActivity {
    active: Arc<AtomicUsize>,
}

impl ExportActivity {
    pub(crate) fn begin(active: Arc<AtomicUsize>) -> Self {
        active.fetch_add(1, Ordering::AcqRel);
        Self { active }
    }
}

impl Drop for ExportActivity {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::AcqRel);
    }
}
