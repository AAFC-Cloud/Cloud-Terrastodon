use crate::ArtifactFreshness;
use crate::ArtifactMetadata;
use crate::ArtifactSet;
use crate::CacheEntryIdentity;
use crate::WorkCache;
use crate::WorkOutput;
use crate::artifact_age::ArtifactAge;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::work_freshness::WorkFreshness;
use uuid::Uuid;

/// Complete raw bytes admitted by the store; the current decoder must accept them.
#[derive(Debug)]
pub struct LoadedArtifact {
    pub artifacts: ArtifactSet,
    pub metadata: ArtifactMetadata,
    pub freshness: ArtifactFreshness,
    // Keep the owner and original admission proof bound to these decoded bytes.
    pub(crate) cache: WorkCache,
    pub(crate) identity: CacheEntryIdentity,
    pub(crate) generation: CacheGenerationStamp,
    pub(crate) age: ArtifactAge,
    pub(crate) work: Option<Uuid>,
}

impl LoadedArtifact {
    /// Attach a canonical decoded value to this load's original age and generations.
    /// Invalidation during decoding makes the result nonreusable at publication.
    /// Typed reuse requires loading through `ArtifactEntry::for_work` for this
    /// same active initializer; an inspection load cannot become a later fill.
    pub fn decoded<T>(self, value: T) -> WorkOutput<T> {
        WorkOutput {
            value,
            freshness: WorkFreshness::Loaded {
                cache: self.cache,
                identity: self.identity,
                generation: self.generation,
                age: self.age,
                work: self.work,
            },
        }
    }
}
