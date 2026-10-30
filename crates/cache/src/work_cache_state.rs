use crate::CacheScope;
use crate::ExecutorKind;
use crate::cache_ownership_claim::CacheOwnershipClaim;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Shared backing ownership retained for the lifetime of all cloned handles.
/// The mutex atomically reserves logical contracts; it does not lock execution.
#[derive(Debug)]
pub(crate) struct WorkCacheState {
    pub root: PathBuf,
    pub claims: Mutex<HashMap<(ExecutorKind, CacheScope), CacheOwnershipClaim>>,
}
