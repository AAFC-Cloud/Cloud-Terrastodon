use crate::CacheEntryIdentity;
use crate::WorkCache;
use crate::artifact_age::ArtifactAge;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::cache_state::CacheState;
use crate::work_certificate::WorkCertificate;
use crate::work_dependency::WorkDependency;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::sync::Arc;
use std::time::Duration;
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
            .filter(|attempt| !attempt.finished)
            .map(|attempt| &attempt.identity)
            .ok_or_else(|| eyre!("cache initializer attempt has already completed"))
    }

    pub(crate) fn mark_nonreusable(&self, state: &mut CacheState) {
        let mut ancestor = Some(self.id);
        while let Some(id) = ancestor {
            let Some(attempt) = state.attempts.get_mut(&id) else {
                break;
            };
            attempt.reusable = false;
            ancestor = attempt.parent;
            if attempt.publication_started {
                let identity = attempt.identity.clone();
                // A leaf selection may already be visible. A later failed child
                // admission or unfinished callback must close disk reuse too.
                state.publication_quarantine.insert(identity);
            }
        }
    }

    /// Freeze the ledger before selecting durable leaf artifacts. The generic
    /// leaf store cannot persist aggregates without authoritative provenance.
    pub(crate) fn admit_leaf(&self, state: &mut CacheState) -> Result<bool> {
        let attempt = state
            .attempts
            .get_mut(&self.id)
            .filter(|attempt| !attempt.finished)
            .ok_or_else(|| eyre!("artifact initializer attempt has already completed"))?;
        attempt.frozen = true;
        Ok(attempt.reusable
            && attempt.pending_children == 0
            && attempt.dependencies.is_empty()
            && state
                .generations
                .matches(&attempt.identity, &attempt.generation))
    }

    pub(crate) fn finish(
        &self,
        state: &mut CacheState,
        age: ArtifactAge,
        validity: Duration,
        loaded_generation: Option<CacheGenerationStamp>,
    ) -> Result<WorkCertificate> {
        let attempt = state
            .attempts
            .get_mut(&self.id)
            .ok_or_else(|| eyre!("cache initializer attempt has already completed"))?;
        attempt.frozen = true;
        attempt.finished = true;
        if loaded_generation
            .as_ref()
            .is_some_and(|stamp| stamp != &attempt.generation)
        {
            attempt.reusable = false;
        }
        let certificate = WorkCertificate {
            own: WorkDependency {
                identity: attempt.identity.clone(),
                generation: attempt.generation.clone(),
                age,
                validity,
            },
            dependencies: attempt.dependencies.clone(),
            reusable: attempt.reusable && attempt.pending_children == 0,
        };
        if !certificate.is_reusable(state) {
            self.mark_nonreusable(state);
        }
        Ok(certificate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactFreshness;
    use crate::ArtifactObservation;
    use crate::AuthPartition;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;
    use crate::cached_work_value::CachedWorkValue;
    use crate::work_attempt_guard::WorkAttemptGuard;
    use crate::work_child_guard::WorkChildGuard;

    #[test]
    fn finished_tokens_cannot_admit_late_work_or_poison_a_published_result() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let cache = WorkCache::new(temporary.path())?;
        let foreign = WorkCache::new(temporary.path().join("independent-owner"))?;
        let identity = CacheEntryIdentity {
            kind: ExecutorKind::InProcess,
            scope: CacheScope::try_from_components(["synthetic", "finished-result"])?,
            auth_partition: AuthPartition::new("synthetic-session")?,
            fingerprint: NonEmptyString::new("synthetic-inputs")?,
        };
        let guard = WorkAttemptGuard::new(&cache, identity.clone(), None)?;
        let attempt = guard.attempt();
        let value = Arc::new(7_u64);
        let certificate = {
            let mut state = cache.lock_state()?;
            assert!(attempt.admit_leaf(&mut state)?);
            // Model a successful leaf commit so a late admission incorrectly
            // poisoning this attempt would also install disk quarantine.
            state
                .attempts
                .get_mut(&attempt.id)
                .unwrap()
                .publication_started = true;
            let observation = ArtifactObservation::now();
            let certificate = attempt.finish(
                &mut state,
                ArtifactAge {
                    freshness: ArtifactFreshness::new(observation.wall, Duration::MAX)?,
                    initial_age: Duration::ZERO,
                    anchored_at: observation.monotonic,
                },
                Duration::MAX,
                None,
            )?;
            state.values.insert(
                identity.clone(),
                CachedWorkValue {
                    value: value.clone(),
                    certificate: certificate.clone(),
                },
            );
            certificate
        };

        // Exercise the interval after publication releases state but before the
        // attempt guard drops. Both entry points must see a completed token.
        for owner in [&cache, &foreign] {
            let error = WorkChildGuard::admit(owner, &attempt, true)
                .err()
                .expect("a finished token cannot admit a child");
            assert!(error.to_string().contains("already completed"));
            let state = cache.lock_state()?;
            assert!(certificate.is_reusable(&state));
            assert!(state.publication_quarantine.is_empty());
            let (retained, _) = state.values[&identity]
                .load::<u64>(&state, Duration::MAX)?
                .expect("late rejected work cannot invalidate the accepted result");
            assert!(Arc::ptr_eq(&value, &retained));
        }
        {
            let mut state = cache.lock_state()?;
            let error = attempt
                .admit_leaf(&mut state)
                .expect_err("a finished token cannot publish another leaf");
            assert!(error.to_string().contains("already completed"));
            assert!(state.publication_quarantine.is_empty());
        }
        drop(guard);
        let state = cache.lock_state()?;
        assert!(state.publication_quarantine.is_empty());
        assert!(state.values[&identity].is_reusable(&state));
        Ok(())
    }
}
