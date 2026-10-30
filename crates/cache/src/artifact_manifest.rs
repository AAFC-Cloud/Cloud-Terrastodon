use crate::ArtifactFreshness;
use crate::ArtifactMetadata;
use crate::CacheEntryIdentity;
use crate::artifact_digest::ArtifactDigest;
use crate::artifact_file_digest::ArtifactFileDigest;
use facet::Facet;
use uuid::Uuid;

/// A completed immutable artifact set. Version describes storage, never Rust T.
#[derive(Debug, Facet)]
pub(crate) struct ArtifactManifest {
    pub version: u8,
    pub identity: CacheEntryIdentity,
    pub attempt: Uuid,
    pub metadata: ArtifactMetadata,
    pub freshness: ArtifactFreshness,
    pub payload_digest: ArtifactDigest,
    pub inputs: Vec<ArtifactFileDigest>,
    pub diagnostics: Vec<ArtifactFileDigest>,
}

impl ArtifactManifest {
    /// Storage format shared by manifests and their selection records.
    pub const VERSION: u8 = 1;
}
