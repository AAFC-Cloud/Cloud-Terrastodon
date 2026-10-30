use crate::CacheEntryIdentity;
use crate::WorkAttempt;
use crate::WorkCache;
use crate::work_attempt_state::WorkAttemptState;
use eyre::Result;
use eyre::ensure;
use uuid::Uuid;

/// Bounded initializer ownership. Drop closes all clones of its ancestry token.
pub(crate) struct WorkAttemptGuard {
    attempt: WorkAttempt,
}

impl WorkAttemptGuard {
    pub fn new(
        cache: &WorkCache,
        identity: CacheEntryIdentity,
        parent: Option<&WorkAttempt>,
    ) -> Result<Self> {
        let id = Uuid::new_v4();
        let mut state = cache.lock_state()?;
        ensure!(
            !state.attempts.contains_key(&id),
            "cache attempt identity collision"
        );
        if let Some(parent) = parent {
            parent.entry(cache, &state)?;
        }
        let generation = state.generations.capture(&identity);
        state.attempts.insert(
            id,
            WorkAttemptState {
                identity,
                generation,
                parent: parent.map(|parent| parent.id),
                dependencies: Vec::new(),
                pending_children: 0,
                reusable: true,
                frozen: false,
                finished: false,
                publication_started: false,
            },
        );
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
        let mut state = self
            .attempt
            .cache
            .state
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state
            .attempts
            .get(&self.attempt.id)
            .is_some_and(|attempt| !attempt.finished)
        {
            self.attempt.mark_nonreusable(&mut state);
        }
        state.attempts.remove(&self.attempt.id);
    }
}
