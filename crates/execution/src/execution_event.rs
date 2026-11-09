use crate::CollectionProgress;
use cloud_terrastodon_cache::CacheEntryIdentity;
use cloud_terrastodon_cache::WorkCacheStatus;

/// Invocation-local lifecycle and initializer progress, never retained in cache values.
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// The caller's bounded execution scope has admitted the request.
    /// Individual work still performs its own authentication admission.
    Admitted,
    Cache {
        identity: CacheEntryIdentity,
        status: WorkCacheStatus,
    },
    Progress(CollectionProgress),
    Completed,
    Failed,
}
