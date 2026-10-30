use crate::ArtifactEntry;
use crate::ArtifactFilePath;
use crate::ArtifactSet;
use crate::CacheEntryIdentity;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_manifest::ArtifactManifest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use uuid::Uuid;

/// Selects one complete immutable artifact set for raw leaf reuse.
#[derive(Debug, Facet)]
pub(crate) struct ArtifactSelection {
    pub version: u8,
    pub identity: CacheEntryIdentity,
    pub attempt: Uuid,
}

impl ArtifactSelection {
    /// Read this entry's selection under its owner's I/O gate. Missing or
    /// malformed records are misses; a conflicting exact entry identity errors.
    pub fn read(entry: &ArtifactEntry) -> Result<Option<Self>> {
        let directory = ArtifactDirectory::new(entry.cache.root(), entry.directory())?;
        if !directory.exists_checked()? {
            return Ok(None);
        }
        let Some(bytes) = directory
            .file(&ArtifactFilePath::new("entry.json")?)?
            .read()?
        else {
            return Ok(None);
        };
        let Ok(actual) = facet_json::from_slice::<CacheEntryIdentity>(&bytes) else {
            return Ok(None);
        };
        ensure!(
            actual == entry.identity,
            "artifact path collision: original entry identities differ"
        );
        let Some(bytes) = directory
            .file(&ArtifactFilePath::new("selection.json")?)?
            .read()?
        else {
            return Ok(None);
        };
        let Ok(selection) = facet_json::from_slice::<Self>(&bytes) else {
            return Ok(None);
        };
        Ok(Some(selection))
    }

    /// Verify and load the complete attempt referenced by this selection.
    /// The caller retains the entry owner's I/O gate throughout the read.
    pub fn load_artifacts(
        &self,
        entry: &ArtifactEntry,
    ) -> Result<Option<(ArtifactManifest, ArtifactSet)>> {
        if self.version != ArtifactManifest::VERSION {
            return Ok(None);
        }
        ensure!(
            self.identity == entry.identity,
            "artifact selection identity does not match the requested entry"
        );
        let directory = ArtifactDirectory::new(entry.cache.root(), entry.directory())?
            .child("attempts")?
            .child(self.attempt.to_string())?;
        if !directory.exists_checked()? {
            return Ok(None);
        }
        let Some(bytes) = directory
            .file(&ArtifactFilePath::new("manifest.json")?)?
            .read()?
        else {
            return Ok(None);
        };
        let Ok(manifest) = facet_json::from_slice::<ArtifactManifest>(&bytes) else {
            return Ok(None);
        };
        ensure!(
            manifest.identity == entry.identity,
            "artifact manifest identity does not match the requested entry"
        );
        if manifest.version != ArtifactManifest::VERSION || manifest.attempt != self.attempt {
            return Ok(None);
        }
        let Some(payload) = directory
            .file(&ArtifactFilePath::new("payload.bin")?)?
            .read()?
        else {
            return Ok(None);
        };
        if !manifest.payload_digest.matches(&payload) {
            return Ok(None);
        }
        if directory
            .child("inputs")?
            .read_verified_files(&manifest.inputs)?
            .is_none()
        {
            return Ok(None);
        }
        let Some(diagnostics) = directory
            .child("diagnostics")?
            .read_verified_files(&manifest.diagnostics)?
        else {
            return Ok(None);
        };
        Ok(Some((
            manifest,
            ArtifactSet {
                payload,
                diagnostics,
            },
        )))
    }

    /// Replace the visible selection. The caller must hold the owner's I/O gate
    /// and install publication quarantine before this first visible mutation.
    pub fn write(&self, entry: &ArtifactEntry) -> Result<()> {
        ArtifactDirectory::new(entry.cache.root(), entry.directory())?
            .file(&ArtifactFilePath::new("selection.json")?)?
            .replace_json(self)
    }
}
