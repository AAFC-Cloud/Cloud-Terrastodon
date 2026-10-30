use crate::ArtifactAttempt;
use crate::ArtifactFilePath;
use crate::ArtifactMetadata;
use crate::ArtifactRetention;
use crate::CacheEntryIdentity;
use crate::ExecutorKind;
use crate::LoadedArtifact;
use crate::WorkCache;
use crate::artifact_age::ArtifactAge;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_io_guard::ArtifactIoGuard;
use crate::artifact_manifest::ArtifactManifest;
use crate::artifact_selection::ArtifactSelection;
use crate::cache_path_component::CachePathComponent;
use crate::cache_state::CacheState;
use crate::stored_artifact::StoredArtifact;
use chrono::Utc;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::MutexGuard;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationScope;

/// A claimed raw artifact entry bound to one explicit cache owner.
/// No canonical decoder, transport execution, or generation policy lives here.
#[derive(Clone, Debug)]
pub struct ArtifactEntry {
    pub(crate) cache: WorkCache,
    pub(crate) identity: CacheEntryIdentity,
    pub(crate) metadata: ArtifactMetadata,
    pub(crate) validity: Duration,
    pub(crate) retention: ArtifactRetention,
}

impl ArtifactEntry {
    /// Inspect this entry's physical location without creating or reading it.
    pub fn directory(&self) -> PathBuf {
        let mut directory = self.cache.root().join("artifacts-v2").join("entries");
        match &self.identity.kind {
            ExecutorKind::Command => directory.push("command"),
            ExecutorKind::Rest => directory.push("rest"),
            ExecutorKind::InProcess => directory.push("in-process"),
            ExecutorKind::Other(name) => {
                directory.push("other");
                directory.push(CachePathComponent::from_identity(name));
            }
        }
        directory.push("scope");
        for component in self.identity.scope.components() {
            directory.push(CachePathComponent::from_identity(component));
        }
        directory.push("variants");
        directory.push(CachePathComponent::from_identity(
            self.identity.auth_partition.as_str(),
        ));
        directory.push(CachePathComponent::from_identity(
            &self.identity.fingerprint,
        ));
        directory
    }

    /// Initialize or verify this entry's identity under its owner's I/O gate.
    pub(crate) fn ensure_directory(&self) -> Result<()> {
        let directory = ArtifactDirectory::new(self.cache.root(), self.directory())?;
        directory.ensure_exists()?;
        let file = directory.file(&ArtifactFilePath::new("entry.json")?)?;
        if let Some(bytes) = file.read()? {
            let actual: CacheEntryIdentity =
                facet_json::from_slice(&bytes).wrap_err("invalid artifact entry identity")?;
            ensure!(
                actual == self.identity,
                "artifact path collision: original entry identities differ"
            );
        } else {
            file.create_json(&self.identity)?;
        }
        Ok(())
    }

    pub(crate) fn state(&self) -> Result<MutexGuard<'_, CacheState>> {
        self.cache
            .state
            .registry
            .lock()
            .map_err(|_| eyre!("cache state was poisoned while coordinating artifacts"))
    }

    /// Stage invocation inputs in a private workspace. Disabled retention never
    /// creates an attempt or writes any bytes, including its inputs or errors.
    pub async fn begin_attempt(
        &self,
        inputs: BTreeMap<ArtifactFilePath, Vec<u8>>,
        cancellation: &CancellationScope,
    ) -> Result<Option<ArtifactAttempt>> {
        if self.retention == ArtifactRetention::Disabled {
            return Ok(None);
        }
        let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
        cancellation.bail_if_cancelled()?;
        let entry = self.clone();
        let (gate, initialized) = gate.run_blocking(move || entry.ensure_directory()).await?;
        initialized?;
        drop(gate);
        cancellation.bail_if_cancelled()?;
        let entry = self.clone();
        // Unique staging is invisible to readers and can safely outlive a dropped
        // caller. It never mutates a selection or another invocation's inputs.
        let attempt =
            tokio::task::spawn_blocking(move || ArtifactAttempt::stage_inputs(entry, inputs))
                .await
                .wrap_err("artifact input staging worker failed")??;
        cancellation.bail_if_cancelled()?;
        Ok(Some(attempt))
    }

    /// Load a complete selected raw set. The current owning decoder must still
    /// accept it; diagnostic Rust type names never gate reuse.
    pub async fn load(&self, cancellation: &CancellationScope) -> Result<Option<LoadedArtifact>> {
        if self.retention == ArtifactRetention::Disabled || self.validity.is_zero() {
            return Ok(None);
        }
        let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
        cancellation.bail_if_cancelled()?;
        if self
            .state()?
            .publication_quarantine
            .contains(&self.identity)
        {
            return Ok(None);
        }
        let entry = self.clone();
        let (_gate, result) = gate
            .run_blocking(move || {
                let Some(selection) = ArtifactSelection::read(&entry)? else {
                    return Ok(None);
                };
                selection.load_artifacts(&entry)
            })
            .await?;
        cancellation.bail_if_cancelled()?;
        let Some((manifest, artifacts)) = result? else {
            return Ok(None);
        };
        if !self.admit_age(&manifest)? {
            return Ok(None);
        }
        let mut freshness = manifest.freshness;
        freshness.expires_at = freshness.effective_expires_at(self.validity)?;
        Ok(Some(LoadedArtifact {
            artifacts,
            metadata: manifest.metadata,
            freshness,
        }))
    }

    pub(crate) fn admit_age(&self, manifest: &ArtifactManifest) -> Result<bool> {
        let wall = Utc::now();
        let monotonic = Instant::now();
        if !manifest.freshness.is_reusable_at(wall, self.validity) {
            return Ok(false);
        }
        let mut state = self.state()?;
        let age = state
            .ages
            .entry((self.identity.clone(), manifest.attempt))
            .or_insert_with(|| ArtifactAge {
                freshness: manifest.freshness.clone(),
                initial_age: (wall - manifest.freshness.observed_at)
                    .to_std()
                    .unwrap_or(Duration::MAX),
                anchored_at: monotonic,
            });
        Ok(age.reusable(&manifest.freshness, self.validity, wall, monotonic))
    }

    /// Select a completed raw leaf artifact. Call only after canonical decoding
    /// succeeds. Failure/obsolete/aggregate artifacts stay unselected.
    ///
    /// Publication is coherent only within this cache owner. Generation and
    /// dependency eligibility will be added at the gated admission point before
    /// adapter cutover; this API does not yet promise invalidation safety.
    pub async fn publish(
        &self,
        stored: &StoredArtifact,
        cancellation: &CancellationScope,
    ) -> Result<bool> {
        if self.retention == ArtifactRetention::Disabled || self.validity.is_zero() {
            return Ok(false);
        }
        ensure!(
            Arc::ptr_eq(&self.cache.state, &stored.entry.cache.state)
                && self.identity == stored.entry.identity,
            "artifact publication requires the same cache owner and entry"
        );
        cancellation
            .deferred(
                "publish a complete artifact selection",
                |_protected| async {
                    let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
                    let entry = self.clone();
                    let selection = ArtifactSelection {
                        version: ArtifactManifest::VERSION,
                        identity: self.identity.clone(),
                        attempt: stored.attempt,
                    };
                    let (gate, checked) = gate
                        .run_blocking(move || {
                            entry.ensure_directory()?;
                            selection.load_artifacts(&entry)
                        })
                        .await?;
                    let Some((manifest, _)) = checked? else {
                        return Err(eyre!("cannot publish an incomplete artifact set"));
                    };
                    if !self.admit_age(&manifest)? {
                        return Ok(false);
                    }
                    // Future generation/dependency checks belong here under the same gate.
                    // Install quarantine BEFORE any visible selection mutation. It is not
                    // an RAII cleanup: an explicitly dropped future must leave it set.
                    self.state()?
                        .publication_quarantine
                        .insert(self.identity.clone());
                    let entry = self.clone();
                    let selection = ArtifactSelection {
                        version: ArtifactManifest::VERSION,
                        identity: self.identity.clone(),
                        attempt: stored.attempt,
                    };
                    let (_gate, written) =
                        gate.run_blocking(move || selection.write(&entry)).await?;
                    written.wrap_err(
                        "artifact selection remains quarantined after publication failure",
                    )?;
                    // Only the awaited success path clears publication quarantine while
                    // holding the worker-returned gate. No invalidation quarantine clears.
                    self.state()?.publication_quarantine.remove(&self.identity);
                    self.admit_age(&manifest)
                },
            )
            .await
    }
}
