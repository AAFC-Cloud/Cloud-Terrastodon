use crate::ArtifactFilePath;
use crate::artifact_digest::ArtifactDigest;
use facet::Facet;

/// A required input or diagnostic file and the BLAKE3 digest of its exact bytes.
///
/// The digest's persisted representation includes its `blake3:` algorithm tag.
#[derive(Clone, Debug, Facet)]
pub(crate) struct ArtifactFileDigest {
    pub path: ArtifactFilePath,
    pub digest: ArtifactDigest,
}

impl ArtifactFileDigest {
    /// Bind a validated artifact path to a BLAKE3 digest of its contents.
    pub fn new(path: ArtifactFilePath, bytes: &[u8]) -> Self {
        Self {
            path,
            digest: ArtifactDigest::new(bytes),
        }
    }
}
