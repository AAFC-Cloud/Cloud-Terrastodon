use crate::CacheEntryIdentity;
use crate::CacheScope;
use crate::ExecutorKind;
use crate::artifact_age::ArtifactAge;
use crate::cache_ownership_claim::CacheOwnershipClaim;
use crate::cached_work_value::CachedWorkValue;
use crate::work_coordination::WorkCoordination;
use crate::work_wait_graph::WorkWaitGraph;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Weak;
use uuid::Uuid;

/// Short-lived synchronous bookkeeping shared by all handles of one owner.
#[derive(Debug, Default)]
pub(crate) struct CacheState {
    pub claims: HashMap<(ExecutorKind, CacheScope), CacheOwnershipClaim>,
    pub coordination: HashMap<CacheEntryIdentity, Weak<WorkCoordination>>,
    pub values: HashMap<CacheEntryIdentity, CachedWorkValue>,
    pub attempts: HashMap<Uuid, CacheEntryIdentity>,
    pub waits: WorkWaitGraph,
    // Only publication quarantine exists here. Future invalidation quarantine
    // must remain distinct and must never be cleared by a successful publication.
    pub publication_quarantine: HashSet<CacheEntryIdentity>,
    pub ages: HashMap<(CacheEntryIdentity, Uuid), ArtifactAge>,
}
