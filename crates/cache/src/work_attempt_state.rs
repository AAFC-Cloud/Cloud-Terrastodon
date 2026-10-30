use crate::CacheEntryIdentity;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::work_dependency::WorkDependency;
use uuid::Uuid;

/// Monotonic dependency ledger owned by one bounded initializer.
#[derive(Debug)]
pub(crate) struct WorkAttemptState {
    pub identity: CacheEntryIdentity,
    pub generation: CacheGenerationStamp,
    pub parent: Option<Uuid>,
    pub dependencies: Vec<WorkDependency>,
    pub pending_children: usize,
    pub reusable: bool,
    pub frozen: bool,
    pub finished: bool,
    pub publication_started: bool,
}
