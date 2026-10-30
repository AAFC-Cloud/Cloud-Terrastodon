use crate::ArtifactEntry;
use crate::ArtifactFreshness;
use crate::artifact_directory::ArtifactDirectory;
use std::path::Path;
use uuid::Uuid;

/// A complete immutable attempt retained for inspection; selection is separate.
#[derive(Debug)]
pub struct StoredArtifact {
    // Private fields bind this artifact to its owner and prevent forged completion.
    pub(crate) entry: ArtifactEntry,
    pub(crate) attempt: Uuid,
    pub(crate) directory: ArtifactDirectory,
    pub(crate) freshness: ArtifactFreshness,
}

impl StoredArtifact {
    pub fn directory(&self) -> &Path {
        self.directory.path()
    }
}
