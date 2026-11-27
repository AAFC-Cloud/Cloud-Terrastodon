use crate::CollectionProgress;
use cloud_terrastodon_cache::CacheEntryIdentity;
use cloud_terrastodon_cache::WorkCacheStatus;
use std::any::Any;
use std::sync::Arc;

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
    /// An initializer's decoded page. A zero index starts or restarts a collection.
    CollectionPage {
        index: usize,
        payload: Arc<dyn Any + Send + Sync>,
    },
    Completed,
    Failed,
}
