use crate::CacheEntryIdentity;
use crate::WorkCache;
use crate::cache_state::CacheState;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::sync::Arc;
use uuid::Uuid;

/// Explicit ancestry of a currently running cache initializer.
/// Cloning this token does not prolong the initializer or its lock ownership.
/// Completed tokens cannot admit children, even if the same key is reused.
#[derive(Clone, Debug)]
pub struct WorkAttempt {
    pub(crate) cache: WorkCache,
    pub(crate) id: Uuid,
}

impl WorkAttempt {
    pub(crate) fn entry<'a>(
        &self,
        cache: &WorkCache,
        state: &'a CacheState,
    ) -> Result<&'a CacheEntryIdentity> {
        ensure!(
            Arc::ptr_eq(&self.cache.state, &cache.state),
            "cache attempt belongs to a different cache owner"
        );
        state
            .attempts
            .get(&self.id)
            .ok_or_else(|| eyre!("cache initializer attempt has already completed"))
    }
}
