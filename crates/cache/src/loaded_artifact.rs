use crate::ArtifactFreshness;
use crate::ArtifactMetadata;
use crate::ArtifactSet;

/// Complete raw bytes admitted by the store; the current decoder must accept them.
#[derive(Debug)]
pub struct LoadedArtifact {
    pub artifacts: ArtifactSet,
    pub metadata: ArtifactMetadata,
    pub freshness: ArtifactFreshness,
}
