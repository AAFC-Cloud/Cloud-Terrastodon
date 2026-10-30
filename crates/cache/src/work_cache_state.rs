use crate::cache_state::CacheState;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

/// Shared backing ownership retained for the lifetime of all cloned handles.
/// The registry mutex protects short bookkeeping changes. The asynchronous gate
/// coordinates artifact I/O; per-entry initializer mutexes coalesce execution.
#[derive(Debug)]
pub(crate) struct WorkCacheState {
    pub root: PathBuf,
    pub registry: Mutex<CacheState>,
    pub artifact_gate: Arc<tokio::sync::Mutex<()>>,
}
