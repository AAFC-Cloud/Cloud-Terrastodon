use crate::ArtifactEntry;
use crate::ArtifactFilePath;
use crate::ArtifactFreshness;
use crate::ArtifactObservation;
use crate::ArtifactSet;
use crate::StoredArtifact;
use crate::artifact_age::ArtifactAge;
use crate::artifact_digest::ArtifactDigest;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_file_digest::ArtifactFileDigest;
use crate::artifact_manifest::ArtifactManifest;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use uuid::Uuid;

/// A unique input workspace that can be sealed once into immutable artifacts.
#[derive(Debug)]
pub struct ArtifactAttempt {
    entry: ArtifactEntry,
    attempt: Uuid,
    directory: ArtifactDirectory,
    inputs_directory: ArtifactDirectory,
    inputs: Vec<ArtifactFileDigest>,
}

impl ArtifactAttempt {
    /// Allocate an exclusive attempt directory and stage all declared inputs.
    pub(crate) fn stage_inputs(
        entry: ArtifactEntry,
        inputs: BTreeMap<ArtifactFilePath, Vec<u8>>,
    ) -> Result<Self> {
        let attempt = Uuid::new_v4();
        let directory = ArtifactDirectory::new(entry.cache.root(), entry.directory())?
            .child("attempts")?
            .create_new_child(attempt.to_string())?;
        let inputs_directory = directory.child("inputs")?;
        inputs_directory.ensure_exists()?;
        let inputs = inputs_directory.write_files(inputs)?;
        Ok(Self {
            entry,
            attempt,
            directory,
            inputs_directory,
            inputs,
        })
    }

    pub fn inputs_directory(&self) -> &Path {
        self.inputs_directory.path()
    }

    /// Locate a declared adjacent input; arbitrary undeclared paths are rejected.
    pub fn input_path(&self, path: &ArtifactFilePath) -> Result<PathBuf> {
        ensure!(
            self.inputs.iter().any(|input| input.path == *path),
            "input was not staged by this artifact attempt"
        );
        Ok(path.join_to(self.inputs_directory.path()))
    }

    /// Retain a complete outcome without selecting it for reuse. Required
    /// artifacts finish in a required-completion scope for the observed outcome. Explicit future
    /// drop can leave an unselected staging directory, never a reusable result.
    pub async fn finish(
        self,
        artifacts: ArtifactSet,
        observation: ArtifactObservation,
        cancellation: &CancellationScope,
    ) -> Result<StoredArtifact> {
        cancellation
            .finish_required(
                "retain artifacts for an observed execution outcome",
                |_protected| async move {
                    let freshness = ArtifactFreshness::new(observation.wall, self.entry.validity)?;
                    let stored = tokio::task::spawn_blocking(move || -> Result<StoredArtifact> {
                        ensure!(
                            self.directory.exists_checked()?,
                            "artifact attempt directory disappeared"
                        );
                        let diagnostics = self
                            .directory
                            .child("diagnostics")?
                            .write_files(artifacts.diagnostics)?;
                        self.directory
                            .file(&ArtifactFilePath::new("payload.bin")?)?
                            .create_new(&artifacts.payload)?;
                        let manifest = ArtifactManifest {
                            version: ArtifactManifest::VERSION,
                            identity: self.entry.identity.clone(),
                            attempt: self.attempt,
                            metadata: self.entry.metadata.clone(),
                            freshness: freshness.clone(),
                            payload_digest: ArtifactDigest::new(&artifacts.payload),
                            inputs: self.inputs,
                            diagnostics,
                        };
                        self.directory
                            .file(&ArtifactFilePath::new("manifest.json")?)?
                            .create_json(&manifest)?;
                        Ok(StoredArtifact {
                            entry: self.entry,
                            attempt: self.attempt,
                            directory: self.directory,
                            freshness,
                        })
                    })
                    .await
                    .wrap_err("artifact outcome staging worker failed")??;
                    stored.entry.state()?.ages.insert(
                        (stored.entry.identity.clone(), stored.attempt),
                        ArtifactAge {
                            freshness: stored.freshness.clone(),
                            initial_age: Duration::ZERO,
                            anchored_at: observation.monotonic,
                        },
                    );
                    Ok(stored)
                },
            )
            .await
    }
}
