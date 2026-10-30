use crate::CacheEntryIdentity;
use crate::WorkCache;
use crate::cache_state::CacheState;
use eyre::Result;

/// Retains one wait edge until the entire nested invocation completes or drops.
pub(crate) struct WorkWaitGuard {
    cache: WorkCache,
    source: CacheEntryIdentity,
    target: CacheEntryIdentity,
}

impl WorkWaitGuard {
    pub fn new(
        cache: &WorkCache,
        state: &mut CacheState,
        source: CacheEntryIdentity,
        target: CacheEntryIdentity,
    ) -> Result<Self> {
        state.waits.add(&source, &target)?;
        Ok(Self {
            cache: cache.clone(),
            source,
            target,
        })
    }
}

impl Drop for WorkWaitGuard {
    fn drop(&mut self) {
        self.cache
            .state
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .waits
            .remove(&self.source, &self.target);
    }
}
