use crate::CacheEntryIdentity;
use crate::WorkAttempt;
use crate::WorkCache;
use eyre::Result;
use eyre::ensure;
use uuid::Uuid;

/// Bounded initializer ownership. Drop closes all clones of its ancestry token.
pub(crate) struct WorkAttemptGuard {
    attempt: WorkAttempt,
}

impl WorkAttemptGuard {
    pub fn new(cache: &WorkCache, identity: CacheEntryIdentity) -> Result<Self> {
        let id = Uuid::new_v4();
        let mut state = cache.lock_state()?;
        ensure!(
            !state.attempts.contains_key(&id),
            "cache attempt identity collision"
        );
        state.attempts.insert(id, identity);
        Ok(Self {
            attempt: WorkAttempt {
                cache: cache.clone(),
                id,
            },
        })
    }

    pub fn attempt(&self) -> WorkAttempt {
        self.attempt.clone()
    }
}

impl Drop for WorkAttemptGuard {
    fn drop(&mut self) {
        // Cleanup cannot report poisoning; later admissions still fail closed.
        self.attempt
            .cache
            .state
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .attempts
            .remove(&self.attempt.id);
    }
}
