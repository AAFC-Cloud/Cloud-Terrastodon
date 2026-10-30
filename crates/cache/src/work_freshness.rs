use crate::ArtifactFreshness;
use crate::ArtifactObservation;
use crate::CacheEntryIdentity;
use crate::WorkAttempt;
use crate::WorkCache;
use crate::artifact_age::ArtifactAge;
use crate::cache_generation_stamp::CacheGenerationStamp;
use eyre::Result;
use eyre::ensure;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Original observation proof; disk decoding must never restamp loaded bytes.
#[derive(Debug)]
pub(crate) enum WorkFreshness {
    Observed(ArtifactObservation),
    Loaded {
        cache: WorkCache,
        identity: CacheEntryIdentity,
        generation: CacheGenerationStamp,
        age: ArtifactAge,
        work: Option<Uuid>,
    },
}

impl WorkFreshness {
    pub fn into_age(
        self,
        cache: &WorkCache,
        identity: &CacheEntryIdentity,
        attempt: &WorkAttempt,
        validity: Duration,
    ) -> Result<(ArtifactAge, Option<CacheGenerationStamp>)> {
        match self {
            Self::Observed(observation) => Ok((
                ArtifactAge {
                    freshness: ArtifactFreshness::new(observation.wall, validity)?,
                    initial_age: Duration::ZERO,
                    anchored_at: observation.monotonic,
                },
                None,
            )),
            Self::Loaded {
                cache: owner,
                identity: original,
                generation,
                age,
                work,
            } => {
                ensure!(
                    Arc::ptr_eq(&cache.state, &owner.state) && *identity == original,
                    "decoded artifact belongs to a different cache owner or entry"
                );
                ensure!(
                    work == Some(attempt.id),
                    "typed artifact reuse requires loading through for_work for the current initializer"
                );
                Ok((age, Some(generation)))
            }
        }
    }
}
