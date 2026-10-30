use crate::cache_state::CacheState;
use crate::work_dependency::WorkDependency;
use std::time::Duration;

/// Frozen proof for one result and all semantic observations it consumed.
///
/// Repeated observations remain separate: neither a newer generation nor a
/// looser caller bound can overwrite an earlier dependency in the same attempt.
#[derive(Clone, Debug)]
pub(crate) struct WorkCertificate {
    pub own: WorkDependency,
    pub dependencies: Vec<WorkDependency>,
    pub reusable: bool,
}

impl WorkCertificate {
    pub fn observations(&self) -> impl Iterator<Item = &WorkDependency> {
        std::iter::once(&self.own).chain(&self.dependencies)
    }

    pub fn is_reusable(&self, state: &CacheState) -> bool {
        self.reusable && self.observations().all(|entry| entry.is_reusable(state))
    }

    pub fn for_caller(&self, validity: Duration) -> Self {
        let mut certificate = self.clone();
        certificate.own.validity = certificate.own.validity.min(validity);
        certificate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactFreshness;
    use crate::AuthPartition;
    use crate::CacheEntryIdentity;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;
    use crate::WorkCache;
    use crate::artifact_age::ArtifactAge;
    use crate::work_attempt_guard::WorkAttemptGuard;
    use crate::work_child_guard::WorkChildGuard;
    use chrono::TimeDelta;
    use chrono::Utc;
    use std::time::Instant;

    fn identity(name: &str) -> CacheEntryIdentity {
        CacheEntryIdentity {
            kind: ExecutorKind::InProcess,
            scope: CacheScope::try_from_components(["synthetic", name]).unwrap(),
            auth_partition: AuthPartition::new("test-session").unwrap(),
            fingerprint: NonEmptyString::new("test-inputs").unwrap(),
        }
    }

    fn age(elapsed: Duration, validity: Duration) -> ArtifactAge {
        ArtifactAge {
            freshness: ArtifactFreshness::new(
                Utc::now() - TimeDelta::from_std(elapsed).unwrap(),
                validity,
            )
            .unwrap(),
            initial_age: Duration::ZERO,
            anchored_at: Instant::now() - elapsed,
        }
    }

    #[test]
    fn fresh_parent_assembly_does_not_renew_a_child_observation() -> eyre::Result<()> {
        let temporary = tempfile::tempdir()?;
        let cache = WorkCache::new(temporary.path())?;
        let parent = WorkAttemptGuard::new(&cache, identity("parent"), None)?;
        let child = WorkAttemptGuard::new(&cache, identity("child"), None)?;
        let validity = Duration::from_secs(60);
        let child_certificate = child.attempt().finish(
            &mut *cache.lock_state()?,
            age(Duration::from_secs(30), validity),
            validity,
            None,
        )?;
        let mut completion = WorkChildGuard::admit(&cache, &parent.attempt(), true)?;
        completion.complete(&mut *cache.lock_state()?, &child_certificate)?;
        let certificate = parent.attempt().finish(
            &mut *cache.lock_state()?,
            age(Duration::ZERO, validity),
            validity,
            None,
        )?;
        assert!(certificate.is_reusable(&*cache.lock_state()?));

        // Forty-five seconds after assembly, the fresh parent is still within
        // its minute, but the original child has exhausted that same bound.
        let wall = Utc::now() + TimeDelta::seconds(45);
        let monotonic = Instant::now() + Duration::from_secs(45);
        assert!(certificate.own.age.reusable(
            &certificate.own.age.freshness,
            certificate.own.validity,
            wall,
            monotonic,
        ));
        assert!(certificate.dependencies.iter().any(|dependency| {
            !dependency.age.reusable(
                &dependency.age.freshness,
                dependency.validity,
                wall,
                monotonic,
            )
        }));
        Ok(())
    }

    #[test]
    fn broader_repeated_child_hits_do_not_erase_a_stricter_consumed_deadline() -> eyre::Result<()> {
        let temporary = tempfile::tempdir()?;
        let cache = WorkCache::new(temporary.path())?;
        let parent = WorkAttemptGuard::new(&cache, identity("parent"), None)?;
        let child = WorkAttemptGuard::new(&cache, identity("child"), None)?;
        let validity = Duration::from_secs(3600);
        let original = child.attempt().finish(
            &mut *cache.lock_state()?,
            age(Duration::from_secs(30), validity),
            validity,
            None,
        )?;
        for caller_bound in [Duration::from_secs(60), validity] {
            let mut completion = WorkChildGuard::admit(&cache, &parent.attempt(), true)?;
            completion.complete(
                &mut *cache.lock_state()?,
                &original.for_caller(caller_bound),
            )?;
        }
        let certificate = parent.attempt().finish(
            &mut *cache.lock_state()?,
            age(Duration::ZERO, validity),
            validity,
            None,
        )?;
        assert!(certificate.is_reusable(&*cache.lock_state()?));

        let wall = Utc::now() + TimeDelta::seconds(45);
        let monotonic = Instant::now() + Duration::from_secs(45);
        assert!(original.own.age.reusable(
            &original.own.age.freshness,
            original.own.validity,
            wall,
            monotonic,
        ));
        assert!(certificate.dependencies.iter().any(|dependency| {
            !dependency.age.reusable(
                &dependency.age.freshness,
                dependency.validity,
                wall,
                monotonic,
            )
        }));
        Ok(())
    }
}
