use crate::CacheEntryIdentity;
use crate::artifact_age::ArtifactAge;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::cache_state::CacheState;
use chrono::Utc;
use std::time::Duration;
use std::time::Instant;

/// One consumed observation, with its original generations and caller bound.
#[derive(Clone, Debug)]
pub(crate) struct WorkDependency {
    pub identity: CacheEntryIdentity,
    pub generation: CacheGenerationStamp,
    pub age: ArtifactAge,
    pub validity: Duration,
}

impl WorkDependency {
    pub fn is_reusable(&self, state: &CacheState) -> bool {
        state.generations.matches(&self.identity, &self.generation)
            && self.age.reusable(
                &self.age.freshness,
                self.validity,
                Utc::now(),
                Instant::now(),
            )
    }
}
