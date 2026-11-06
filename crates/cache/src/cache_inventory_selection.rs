use crate::ArtifactFilePath;
use crate::ArtifactFreshness;
use crate::CacheEntryIdentity;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_manifest::ArtifactManifest;
use crate::artifact_selection::ArtifactSelection;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// Selected artifact metadata as observed by an inventory snapshot.
///
/// A selected record does not establish that its payload is present, intact, or
/// reusable. Invalid metadata remains visible so callers can invalidate its
/// canonical entry without first repairing the damaged selector or manifest.
#[derive(Clone, Debug, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum CacheInventorySelection {
    /// The entry has no selector; immutable attempts may still remain.
    Absent,
    /// The selector and manifest agree on identity, version, and attempt.
    Selected(ArtifactFreshness),
    /// The selector or its selected manifest cannot be read or verified.
    Invalid { message: String },
}

impl CacheInventorySelection {
    pub(crate) fn inspect(directory: &ArtifactDirectory, identity: &CacheEntryIdentity) -> Self {
        match Self::read_metadata(directory, identity) {
            Ok(selection) => selection,
            Err(error) => Self::Invalid {
                message: format!("{error:#}"),
            },
        }
    }

    fn read_metadata(directory: &ArtifactDirectory, identity: &CacheEntryIdentity) -> Result<Self> {
        let Some(bytes) = directory
            .file(&ArtifactFilePath::new("selection.json")?)?
            .read()?
        else {
            return Ok(Self::Absent);
        };
        let selection: ArtifactSelection =
            facet_json::from_slice(&bytes).wrap_err("invalid cache inventory selection")?;
        ensure!(
            selection.identity == *identity && selection.version == ArtifactManifest::VERSION,
            "cache inventory selection has an incompatible identity or version"
        );
        let attempt = directory
            .child("attempts")?
            .child(selection.attempt.to_string())?;
        let bytes = attempt
            .file(&ArtifactFilePath::new("manifest.json")?)?
            .read()?
            .ok_or_eyre("selected cache inventory manifest is missing")?;
        let manifest: ArtifactManifest =
            facet_json::from_slice(&bytes).wrap_err("invalid selected cache inventory manifest")?;
        ensure!(
            manifest.identity == *identity
                && manifest.attempt == selection.attempt
                && manifest.version == ArtifactManifest::VERSION,
            "selected cache inventory manifest does not match its entry"
        );
        Ok(Self::Selected(manifest.freshness))
    }
}
