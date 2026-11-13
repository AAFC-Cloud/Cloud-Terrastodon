use crate::cache_root_lease::CacheRootLease;
use crate::cache_state::CacheState;
use crate::work_cache_root::WorkCacheRoot;
use std::sync::Arc;
use std::sync::Mutex;

/// Shared backing ownership retained for the lifetime of all cloned handles.
/// The registry mutex protects short bookkeeping changes. The asynchronous gate
/// coordinates artifact I/O; per-entry initializer mutexes coalesce execution.
#[derive(Debug)]
pub(crate) struct WorkCacheState {
    pub registry: Mutex<CacheState>,
    pub artifact_gate: Arc<tokio::sync::Mutex<()>>,
    // Drop cached values before removing an owned temporary directory. Their
    // destructors may still need files beneath this root.
    pub root: WorkCacheRoot,
    // Release ownership last, after values and temporary-directory cleanup. A
    // successful first acquisition is shared by every clone for this lifetime.
    pub ownership: tokio::sync::OnceCell<CacheRootLease>,
}

impl Drop for WorkCacheState {
    fn drop(&mut self) {
        // Arbitrary cached-value destructors may still use the root. If one
        // panics, or cache bookkeeping was poisoned, keep the recovery marker.
        if let Ok(state) = self.registry.get_mut() {
            drop(std::mem::take(&mut state.values));
            if !std::thread::panicking()
                && state.invalidations.is_empty()
                && state.publication_quarantine.is_empty()
                && let Some(lease) = self.ownership.get()
                && let Err(error) = lease.finish()
            {
                eprintln!("Unable to finish cache ownership; recovery is required: {error:#}");
            }
        }
    }
}
