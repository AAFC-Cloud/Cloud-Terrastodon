use crate::ArtifactAttempt;
use crate::ArtifactFilePath;
use crate::ArtifactMetadata;
use crate::ArtifactRetention;
use crate::CacheEntryIdentity;
use crate::LoadedArtifact;
use crate::WorkAttempt;
use crate::WorkCache;
use crate::artifact_age::ArtifactAge;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_io_guard::ArtifactIoGuard;
use crate::artifact_manifest::ArtifactManifest;
use crate::artifact_selection::ArtifactSelection;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::cache_state::CacheState;
use crate::stored_artifact::StoredArtifact;
use chrono::Utc;
use eyre::Context;
use eyre::OptionExt;
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
/// Canonical decoding and transport execution remain owned by the operation.
#[derive(Clone, Debug)]
pub struct ArtifactEntry {
    pub(crate) cache: WorkCache,
    pub(crate) identity: CacheEntryIdentity,
    pub(crate) metadata: ArtifactMetadata,
    pub(crate) validity: Duration,
    pub(crate) retention: ArtifactRetention,
    pub(crate) work: Option<WorkAttempt>,
}

impl ArtifactEntry {
    /// Bind raw leaf artifacts to the enclosing canonical initializer.
    /// Publication freezes that attempt's child ledger and refuses durable reuse
    /// when semantic dependencies make its output an assembled result.
    pub fn for_work(mut self, attempt: &WorkAttempt) -> Result<Self> {
        {
            let state = self.state()?;
            ensure!(
                attempt.entry(&self.cache, &state)? == &self.identity,
                "raw artifacts must belong to the same concrete work entry"
            );
        }
        self.work = Some(attempt.clone());
        Ok(self)
    }

    /// Inspect this entry's physical location without creating or reading it.
    pub fn directory(&self) -> Result<PathBuf> {
        Ok(
            ArtifactDirectory::for_entry(self.cache.root(), &self.identity)?
                .path()
                .to_path_buf(),
        )
    }

    /// Initialize or verify this entry's identity under its owner's I/O gate.
    pub(crate) fn ensure_directory(&self) -> Result<()> {
        let directory = ArtifactDirectory::new(self.cache.root(), self.directory()?)?;
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
        let generation = {
            let state = self.state()?;
            if let Some(attempt) = &self.work {
                attempt.entry(&self.cache, &state)?;
                state
                    .attempts
                    .get(&attempt.id)
                    .ok_or_eyre("artifact initializer attempt has already completed")?
                    .generation
                    .clone()
            } else {
                state.generations.capture(&self.identity)
            }
        };
        let entry = self.clone();
        let (gate, initialized) = gate.run_blocking(move || entry.ensure_directory()).await?;
        initialized?;
        drop(gate);
        cancellation.bail_if_cancelled()?;
        let entry = self.clone();
        // Unique staging is invisible to readers and can safely outlive a dropped
        // caller. It never mutates a selection or another invocation's inputs.
        let attempt = tokio::task::spawn_blocking(move || {
            ArtifactAttempt::stage_inputs(entry, generation, inputs)
        })
        .await
        .wrap_err("artifact input staging worker failed")??;
        cancellation.bail_if_cancelled()?;
        Ok(Some(attempt))
    }

    /// Load a complete selected raw set. The current owning decoder must still
    /// accept it; diagnostic Rust type names never gate reuse. Bind with
    /// [`Self::for_work`] before loading to import its decoded value into that
    /// initializer. An unbound load is only a raw inspection snapshot.
    pub async fn load(&self, cancellation: &CancellationScope) -> Result<Option<LoadedArtifact>> {
        if self.retention == ArtifactRetention::Disabled || self.validity.is_zero() {
            return Ok(None);
        }
        let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
        cancellation.bail_if_cancelled()?;
        let generation = {
            let state = self.state()?;
            if let Some(attempt) = &self.work {
                attempt.entry(&self.cache, &state)?;
            }
            if state.publication_quarantine.contains(&self.identity)
                || state
                    .invalidations
                    .values()
                    .any(|target| target.covers(&self.identity))
            {
                return Ok(None);
            }
            let generation = state.generations.capture(&self.identity);
            if !state.generations.matches(&self.identity, &generation) {
                return Ok(None);
            }
            generation
        };
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
        let mut age = self
            .state()?
            .ages
            .get(&(self.identity.clone(), manifest.attempt))
            .ok_or_eyre("admitted artifact age is missing its monotonic correlation")?
            .clone();
        // The caller's narrower deadline belongs to this returned proof. Keep
        // the original monotonic anchor and leave other callers' bounds intact.
        age.freshness = freshness.clone();
        Ok(Some(LoadedArtifact {
            artifacts,
            metadata: manifest.metadata,
            freshness,
            cache: self.cache.clone(),
            identity: self.identity.clone(),
            generation,
            age,
            work: self.work.as_ref().map(|attempt| attempt.id),
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
    /// Publication is coherent only within this cache owner. The generation
    /// captured before staging must remain current; invalidation quarantine can
    /// only be resolved by a covering successful invalidation.
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
                && self.identity == stored.entry.identity
                && self.work.as_ref().map(|attempt| attempt.id)
                    == stored.entry.work.as_ref().map(|attempt| attempt.id),
            "artifact publication requires the same cache owner, entry, and work attempt"
        );
        cancellation
            .deferred(
                "publish a complete artifact selection",
                |_protected| async {
                    let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
                    if !self.admit_generation(&stored.generation)? {
                        return Ok(false);
                    }
                    if let Some(attempt) = &self.work {
                        let mut state = self.state()?;
                        if !attempt.admit_leaf(&mut state)? {
                            return Ok(false);
                        }
                    }
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
                    // Install quarantine BEFORE any visible selection mutation. It is not
                    // an RAII cleanup: an explicitly dropped future must leave it set.
                    {
                        let mut state = self.state()?;
                        if let Some(attempt) = &self.work {
                            state
                                .attempts
                                .get_mut(&attempt.id)
                                .ok_or_else(|| {
                                    eyre!("artifact initializer attempt has already completed")
                                })?
                                .publication_started = true;
                        }
                        state.publication_quarantine.insert(self.identity.clone());
                    }
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
                    // A concurrently rejected child may have poisoned the bound
                    // attempt while the filesystem worker ran. Its failure must
                    // not be cleared by this otherwise successful write.
                    {
                        let mut state = self.state()?;
                        if let Some(attempt) = &self.work
                            && !attempt.admit_leaf(&mut state)?
                        {
                            return Ok(false);
                        }
                        // Only awaited success clears publication quarantine;
                        // invalidation quarantine has a separate recovery path.
                        state.publication_quarantine.remove(&self.identity);
                    }
                    self.admit_age(&manifest)
                },
            )
            .await
    }

    fn admit_generation(&self, generation: &CacheGenerationStamp) -> Result<bool> {
        let state = self.state()?;
        Ok(state.generations.matches(&self.identity, generation)
            && !state
                .invalidations
                .values()
                .any(|target| target.covers(&self.identity)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactObservation;
    use crate::ArtifactSet;
    use crate::AuthPartition;
    use crate::CacheKey;
    use crate::CacheOperationOwner;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;
    use crate::work_attempt_guard::WorkAttemptGuard;
    use chrono::TimeDelta;
    use std::future::Future;
    use std::future::poll_fn;
    use std::pin::Pin;
    use std::task::Poll;
    use teamy_cancellation::CancellationToken;
    use tokio::time::timeout;

    struct SyntheticRead;

    #[tokio::test]
    async fn publication_and_invalidation_obey_the_order_of_the_shared_io_gate() -> Result<()> {
        async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
            poll_fn(|context| {
                assert!(future.as_mut().poll(context).is_pending());
                Poll::Ready(())
            })
            .await;
        }

        CancellationToken::new()
            .with_async_scope(|scope| async move {
                for publication_first in [true, false] {
                    let temporary = tempfile::tempdir()?;
                    let cache = WorkCache::new(temporary.path())?;
                    let entry = cache.artifacts::<String>(
                        ExecutorKind::Rest,
                        CacheKey {
                            scope: CacheScope::try_from_components(["synthetic", "gate-order"])?,
                            valid_for: Duration::from_secs(3600),
                        },
                        CacheOperationOwner::of::<SyntheticRead>(),
                        AuthPartition::new("synthetic-session")?,
                        NonEmptyString::new("query")?,
                        ArtifactRetention::Full,
                    )?;
                    let stored = entry
                        .begin_attempt(BTreeMap::new(), &scope)
                        .await?
                        .unwrap()
                        .finish(
                            ArtifactSet {
                                payload: b"synthetic result".to_vec(),
                                diagnostics: BTreeMap::new(),
                            },
                            ArtifactObservation::now(),
                            &scope,
                        )
                        .await?;
                    let gate = ArtifactIoGuard::acquire(&cache.state.artifact_gate).await;
                    let publication = entry.publish(&stored, &scope);
                    let invalidation = cache.invalidate(&entry.identity.scope, &scope);
                    tokio::pin!(publication, invalidation);

                    // Queue both operations while the gate is held. Its FIFO
                    // ordering makes each interleaving explicit without sleeps.
                    if publication_first {
                        assert_pending(publication.as_mut()).await;
                        assert_pending(invalidation.as_mut()).await;
                    } else {
                        assert_pending(invalidation.as_mut()).await;
                        assert_pending(publication.as_mut()).await;
                    }
                    drop(gate);
                    let (published, invalidated) = timeout(Duration::from_secs(10), async {
                        tokio::join!(publication, invalidation)
                    })
                    .await?;
                    invalidated?;
                    assert_eq!(published?, publication_first);
                    // Publication that wins is removed by invalidation; a
                    // candidate that loses never becomes a reusable selection.
                    assert!(entry.load(&scope).await?.is_none());
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn decoded_disk_proof_preserves_narrower_expiry_and_original_monotonic_anchor()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let temporary = tempfile::tempdir()?;
                let cache = WorkCache::new(temporary.path())?;
                let entry = cache.artifacts::<String>(
                    ExecutorKind::Rest,
                    CacheKey {
                        scope: CacheScope::try_from_components(["synthetic", "deadline"])?,
                        valid_for: Duration::from_secs(60),
                    },
                    CacheOperationOwner::of::<SyntheticRead>(),
                    AuthPartition::new("synthetic-session")?,
                    NonEmptyString::new("query")?,
                    ArtifactRetention::Full,
                )?;
                let stored = entry
                    .begin_attempt(BTreeMap::new(), &scope)
                    .await?
                    .unwrap()
                    .finish(
                        ArtifactSet {
                            payload: b"value".to_vec(),
                            diagnostics: BTreeMap::new(),
                        },
                        ArtifactObservation::now(),
                        &scope,
                    )
                    .await?;
                assert!(entry.publish(&stored, &scope).await?);
                let original = entry
                    .state()?
                    .ages
                    .get(&(entry.identity.clone(), stored.attempt))
                    .unwrap()
                    .clone();
                let work = WorkAttemptGuard::new(&cache, entry.identity.clone(), None)?;
                let narrow = ArtifactEntry {
                    validity: Duration::from_secs(10),
                    ..entry.clone()
                }
                .for_work(&work.attempt())?;
                let loaded = narrow.load(&scope).await?.unwrap();
                let output = loaded.decoded(String::from("value"));
                let (age, _) = output.freshness.into_age(
                    &cache,
                    &entry.identity,
                    &work.attempt(),
                    Duration::MAX,
                )?;
                assert_eq!(age.anchored_at, original.anchored_at);
                assert_eq!(age.initial_age, original.initial_age);
                assert_eq!(
                    age.freshness.expires_at,
                    Some(original.freshness.observed_at + TimeDelta::seconds(10))
                );
                assert!(!age.reusable(
                    &age.freshness,
                    Duration::MAX,
                    age.freshness.observed_at + TimeDelta::seconds(5),
                    age.anchored_at + Duration::from_secs(11)
                ));
                // A narrower caller never shortens another caller's stored proof.
                assert_eq!(
                    entry
                        .state()?
                        .ages
                        .get(&(entry.identity.clone(), stored.attempt))
                        .unwrap()
                        .freshness,
                    original.freshness
                );
                Ok(())
            })
            .await
    }
}
