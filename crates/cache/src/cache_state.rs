use crate::CacheEntryIdentity;
use crate::CacheScope;
use crate::ExecutorKind;
use crate::artifact_age::ArtifactAge;
use crate::cache_ownership_claim::CacheOwnershipClaim;
use std::collections::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

/// Short-lived synchronous bookkeeping shared by all handles of one owner.
#[derive(Debug, Default)]
pub(crate) struct CacheState {
    pub claims: HashMap<(ExecutorKind, CacheScope), CacheOwnershipClaim>,
    // Only publication quarantine exists here. Future invalidation quarantine
    // must remain distinct and must never be cleared by a successful publication.
    pub publication_quarantine: HashSet<CacheEntryIdentity>,
    pub ages: HashMap<(CacheEntryIdentity, Uuid), ArtifactAge>,
}
