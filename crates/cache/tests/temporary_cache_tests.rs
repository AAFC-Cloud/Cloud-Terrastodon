#![cfg(test)]

use cloud_terrastodon_cache::ArtifactFilePath;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheEntryIdentity;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationToken;

struct SyntheticRead;

struct RootObservation {
    root: PathBuf,
    existed_during_drop: Arc<AtomicBool>,
}

impl Drop for RootObservation {
    fn drop(&mut self) {
        self.existed_during_drop
            .store(self.root.exists(), Ordering::SeqCst);
    }
}

#[test]
fn transferred_temporary_directory_survives_until_the_last_cache_clone_drops() -> eyre::Result<()> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().to_path_buf();
    let contents = root.join("owned-input.txt");
    fs::write(&contents, b"synthetic retained contents")?;

    let cache = WorkCache::from_tempdir(temporary)?;
    let cloned = cache.clone();
    assert_eq!(cache.root(), root);
    drop(cache);
    assert_eq!(fs::read(&contents)?, b"synthetic retained contents");

    drop(cloned);
    assert!(!root.exists());
    Ok(())
}

#[test]
fn path_backed_cache_leaves_directory_cleanup_with_its_caller() -> eyre::Result<()> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().to_path_buf();
    let contents = root.join("caller-owned.txt");
    fs::write(&contents, b"caller-owned contents")?;

    let cache = WorkCache::new(&root)?;
    drop(cache);
    assert_eq!(fs::read(&contents)?, b"caller-owned contents");

    drop(temporary);
    assert!(!root.exists());
    Ok(())
}

#[tokio::test]
async fn artifact_attempt_and_stored_artifact_retain_the_temporary_root() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let entry = cache.artifacts::<String>(
                ExecutorKind::Rest,
                CacheKey {
                    scope: CacheScope::try_from_components(["synthetic", "temporary-root"])?,
                    valid_for: Duration::from_secs(60),
                },
                CacheOperationOwner::of::<SyntheticRead>(),
                AuthPartition::new("synthetic-session")?,
                NonEmptyString::new("synthetic-inputs")?,
                ArtifactRetention::Full,
            )?;
            let input = ArtifactFilePath::new("input.txt")?;
            let attempt = entry
                .begin_attempt(
                    BTreeMap::from([(input.clone(), b"synthetic input".to_vec())]),
                    &scope,
                )
                .await?
                .expect("full retention stages the input");
            let input_path = attempt.input_path(&input)?;

            drop(entry);
            drop(cache);
            assert_eq!(fs::read(&input_path)?, b"synthetic input");

            let stored = attempt
                .finish(
                    ArtifactSet {
                        payload: b"synthetic output".to_vec(),
                        diagnostics: BTreeMap::new(),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert_eq!(
                fs::read(stored.directory().join("payload.bin"))?,
                b"synthetic output"
            );
            assert_eq!(fs::read(input_path)?, b"synthetic input");

            drop(stored);
            assert!(!root.exists());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cached_values_are_destroyed_before_the_temporary_directory() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let existed_during_drop = Arc::new(AtomicBool::new(false));
            let observation = RootObservation {
                root: root.clone(),
                existed_during_drop: existed_during_drop.clone(),
            };
            let value = cache
                .cached_work(
                    CacheEntryIdentity {
                        kind: ExecutorKind::InProcess,
                        scope: CacheScope::try_from_components(["synthetic", "drop-order"])?,
                        auth_partition: AuthPartition::new("synthetic-session")?,
                        fingerprint: NonEmptyString::new("synthetic-inputs")?,
                    },
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    Duration::from_secs(60),
                    None,
                    &scope,
                    move |_, _| async move {
                        Ok(WorkOutput::new(observation, ArtifactObservation::now()))
                    },
                )?
                .await?;
            drop(value);
            assert!(!existed_during_drop.load(Ordering::SeqCst));

            drop(cache);
            assert!(existed_during_drop.load(Ordering::SeqCst));
            assert!(!root.exists());
            Ok(())
        })
        .await
}
