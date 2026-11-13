use crate::ArtifactFilePath;
use crate::WorkCache;
use crate::artifact_directory::ArtifactDirectory;
use crate::cache_invalidation_target::CacheInvalidationTarget;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use same_file::Handle;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;
use teamy_cancellation::CancellationScope;

/// Exclusive ownership of a local cache root until its final shared owner drops.
///
/// The lock file is permanent: unlinking it could let another process lock a
/// different file at the same path. All cooperating readers must use this lease.
/// The active marker survives process termination and failed cache completion;
/// recovery discards reusable selectors while retaining immutable diagnostics.
/// This is process-crash recovery, not a power-loss durability guarantee.
#[derive(Debug)]
pub(crate) struct CacheRootLease {
    directory: ArtifactDirectory,
    // Never explicitly unlock this file or acquire locks through its duplicates.
    // A short-lived duplicate only checks physical identity. Closing the final
    // handle releases its OS lock, including on process termination.
    lock: File,
}

impl CacheRootLease {
    const LOCK_FILE: &str = ".owner.lock";
    const ACTIVE_FILE: &str = ".owner-active";

    pub async fn acquire(cache: &WorkCache, cancellation: &CancellationScope) -> Result<Self> {
        let owner = cache.clone();
        let (directory, lock) = tokio::task::spawn_blocking(move || Self::open(owner.root()))
            .await
            .wrap_err("cache ownership file worker failed")??;
        let mut warned = false;
        loop {
            // Preparation and each contention retry are separate cancellation
            // boundaries. try_lock never leaves a detached blocking waiter.
            cancellation.bail_if_cancelled()?;
            match lock.try_lock() {
                Ok(()) => break,
                Err(TryLockError::WouldBlock) => {
                    if !warned {
                        tracing::warn!(
                            "waiting for exclusive cache root ownership; another process may be active"
                        );
                        warned = true;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(TryLockError::Error(error)) => {
                    return Err(error).wrap_err("acquiring exclusive cache root ownership");
                }
            }
        }
        let lease = Self { directory, lock };
        // A dropped awaiter cannot release the lock while recovery still removes
        // selectors. The worker owns the lease and leaves the marker on failure.
        let owner = cache.clone();
        tokio::task::spawn_blocking(move || {
            lease.recover()?;
            drop(owner);
            Ok(lease)
        })
        .await
        .wrap_err("cache ownership recovery worker failed")?
    }

    fn open(root: &Path) -> Result<(ArtifactDirectory, File)> {
        let directory = ArtifactDirectory::new(root, root)?;
        directory.ensure_exists()?;
        let path = root.join(Self::LOCK_FILE);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "cache ownership lock must be a regular file, not a link"
            ),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error).wrap_err("inspecting cache ownership lock"),
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .wrap_err("opening cache ownership lock")?;
        Ok((directory, file))
    }

    fn recover(&self) -> Result<()> {
        self.verify_lock_identity()?;
        let marker = self
            .directory
            .file(&ArtifactFilePath::new(Self::ACTIVE_FILE)?)?;
        if marker.read()?.is_some() {
            ArtifactDirectory::invalidate_selections(
                self.directory.root(),
                &CacheInvalidationTarget::All,
            )
            .wrap_err("recovering an interrupted cache owner; reuse remains disabled")?;
        } else {
            // Establish the marker before admitting any work. An empty/partial
            // marker also means interrupted ownership; its contents are diagnostic.
            marker.create_new(b"cache-root-ownership-v1\n")?;
        }
        Ok(())
    }

    /// Called only after cached values have dropped and bookkeeping is healthy.
    /// Failure deliberately leaves the marker for the next owner to recover.
    pub fn finish(&self) -> Result<()> {
        self.verify_lock_identity()?;
        self.directory
            .file(&ArtifactFilePath::new(Self::ACTIVE_FILE)?)?
            .remove()
    }

    fn verify_lock_identity(&self) -> Result<()> {
        // A temporary owner may remove its root while a waiter holds an open
        // lock handle. Locking that unlinked file would not exclude a new owner
        // using a recreated root. Check both physical files while handles stay
        // open, before touching any selectors or ownership marker.
        let held = Handle::from_file(
            self.lock
                .try_clone()
                .wrap_err("duplicating cache ownership handle for identity verification")?,
        )
        .wrap_err("identifying held cache ownership file")?;
        let current = Handle::from_path(self.directory.root().join(Self::LOCK_FILE))
            .wrap_err("identifying current cache ownership file; the root may have been removed")?;
        ensure!(
            held == current,
            "cache ownership file was replaced while this owner held its handle"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CacheRootLease;
    use crate::ArtifactEntry;
    use crate::ArtifactObservation;
    use crate::ArtifactRetention;
    use crate::ArtifactSet;
    use crate::AuthPartition;
    use crate::CacheKey;
    use crate::CacheOperationOwner;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;
    use crate::WorkCache;
    use crate::artifact_io_guard::ArtifactIoGuard;
    use eyre::Result;
    use std::collections::BTreeMap;
    use std::fs;
    use std::fs::OpenOptions;
    use std::fs::TryLockError;
    use std::sync::mpsc;
    use std::time::Duration;
    use teamy_cancellation::CancellationScope;
    use teamy_cancellation::CancellationToken;
    use tokio::sync::oneshot;
    use tokio::time::timeout;

    const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

    struct SyntheticRead;

    fn entry(cache: &WorkCache, name: &str) -> Result<ArtifactEntry> {
        cache.artifacts::<String>(
            ExecutorKind::Rest,
            CacheKey {
                scope: CacheScope::try_from_components(["synthetic", name])?,
                valid_for: Duration::from_secs(3600),
            },
            CacheOperationOwner::of::<SyntheticRead>(),
            AuthPartition::new("synthetic-session")?,
            NonEmptyString::new("synthetic-query")?,
            ArtifactRetention::Full,
        )
    }

    async fn seed(entry: &ArtifactEntry, scope: &CancellationScope) -> Result<()> {
        let stored = entry
            .begin_attempt(BTreeMap::new(), scope)
            .await?
            .unwrap()
            .finish(
                ArtifactSet {
                    payload: b"synthetic result".to_vec(),
                    diagnostics: BTreeMap::new(),
                },
                ArtifactObservation::now(),
                scope,
            )
            .await?;
        assert!(entry.publish(&stored, scope).await?);
        Ok(())
    }

    #[tokio::test]
    async fn a_replaced_lock_cannot_recover_or_finish_the_new_owners_session() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let temporary = tempfile::tempdir()?;
                let (directory, lock) = CacheRootLease::open(temporary.path())?;
                lock.try_lock()?;
                let previous = CacheRootLease { directory, lock };
                previous.recover()?;

                // Retaining the old open file while replacing its directory entry
                // models a waiter whose temporary root was removed and recreated.
                // An OS lock on the old file cannot protect the replacement root.
                fs::rename(
                    temporary.path().join(CacheRootLease::LOCK_FILE),
                    temporary.path().join("previous-owner.lock"),
                )?;
                let current = WorkCache::new(temporary.path())?;
                timeout(DEADLOCK_GUARD, current.acquire(&scope)).await??;
                let artifact = entry(&current, "replacement-owner")?;
                seed(&artifact, &scope).await?;
                let selection = artifact.directory()?.join("selection.json");
                let selected = fs::read(&selection)?;
                let marker = temporary.path().join(CacheRootLease::ACTIVE_FILE);
                let active = fs::read(&marker)?;

                let recovery_error = previous.recover().unwrap_err();
                let finish_error = previous.finish().unwrap_err();
                assert!(
                    recovery_error
                        .to_string()
                        .contains("ownership file was replaced")
                );
                assert!(
                    finish_error
                        .to_string()
                        .contains("ownership file was replaced")
                );
                assert_eq!(fs::read(selection)?, selected);
                assert_eq!(fs::read(&marker)?, active);
                assert!(artifact.load(&scope).await?.is_some());

                drop(previous);
                drop(artifact);
                drop(current);
                assert!(!marker.exists());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn abandoned_artifact_awaiter_retains_root_ownership_until_its_worker_finishes()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let temporary = tempfile::tempdir()?;
                let cache = WorkCache::new(temporary.path())?;
                cache.acquire(&scope).await?;
                let gate = ArtifactIoGuard::acquire(&cache.state.artifact_gate).await;
                let (started, ready) = oneshot::channel();
                let (release, blocked) = mpsc::channel();
                let (finished, completed) = oneshot::channel();
                let worker_cache = cache.clone();
                let awaiting = tokio::spawn(async move {
                    gate.run_blocking(move || {
                        let _ = started.send(());
                        blocked.recv_timeout(DEADLOCK_GUARD)?;
                        fs::write(worker_cache.root().join("worker-result"), b"finished")?;
                        drop(worker_cache);
                        let _ = finished.send(());
                        Ok(())
                    })
                    .await
                });
                timeout(DEADLOCK_GUARD, ready).await??;
                awaiting.abort();
                assert!(awaiting.await.unwrap_err().is_cancelled());
                drop(cache);

                // This probes the actual OS lock, without depending on how quickly
                // another asynchronous acquisition gets scheduled. The worker's
                // captured owner is now the final handle keeping it held.
                let lock = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(temporary.path().join(CacheRootLease::LOCK_FILE))?;
                assert!(matches!(lock.try_lock(), Err(TryLockError::WouldBlock)));
                drop(lock);
                release.send(())?;
                timeout(DEADLOCK_GUARD, completed).await??;

                let next = WorkCache::new(temporary.path())?;
                timeout(DEADLOCK_GUARD, next.acquire(&scope)).await??;
                assert_eq!(fs::read(next.root().join("worker-result"))?, b"finished");
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn unresolved_completion_requires_recovery_and_failed_recovery_stays_closed() -> Result<()>
    {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                for publication_failure in [false, true] {
                    let temporary = tempfile::tempdir()?;
                    let cache = WorkCache::new(temporary.path())?;
                    let damaged = entry(&cache, "damaged")?;
                    let unrelated = entry(&cache, "unrelated")?;
                    seed(&damaged, &scope).await?;
                    seed(&unrelated, &scope).await?;
                    let selection = damaged.directory()?.join("selection.json");
                    let previous = damaged.directory()?.join("previous-selection.json");
                    fs::rename(&selection, &previous)?;
                    fs::create_dir(&selection)?;

                    // A directory at a selector path creates a deterministic,
                    // portable filesystem failure without permission changes.
                    if publication_failure {
                        let stored = damaged
                            .begin_attempt(BTreeMap::new(), &scope)
                            .await?
                            .unwrap()
                            .finish(
                                ArtifactSet {
                                    payload: b"replacement".to_vec(),
                                    diagnostics: BTreeMap::new(),
                                },
                                ArtifactObservation::now(),
                                &scope,
                            )
                            .await?;
                        assert!(damaged.publish(&stored, &scope).await.is_err());
                    } else {
                        assert!(
                            cache
                                .invalidate(&damaged.identity.scope, &scope)
                                .await
                                .is_err()
                        );
                    }
                    assert!(damaged.load(&scope).await?.is_none());
                    assert!(unrelated.load(&scope).await?.is_some());
                    drop(damaged);
                    drop(unrelated);
                    drop(cache);
                    assert!(temporary.path().join(CacheRootLease::ACTIVE_FILE).is_file());

                    let next = WorkCache::new(temporary.path())?;
                    let error = next.acquire(&scope).await.unwrap_err();
                    assert!(format!("{error:#}").contains("recovering an interrupted cache owner"));
                    assert!(next.state.ownership.get().is_none());
                    assert!(temporary.path().join(CacheRootLease::ACTIVE_FILE).is_file());
                    // A failed recovery must release its physical lock, while
                    // preserving the marker which prohibits future reuse.
                    let lock = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(temporary.path().join(CacheRootLease::LOCK_FILE))?;
                    lock.try_lock()?;
                    drop(lock);

                    fs::remove_dir(&selection)?;
                    fs::rename(&previous, &selection)?;
                    timeout(DEADLOCK_GUARD, next.acquire(&scope)).await??;
                    assert!(entry(&next, "damaged")?.load(&scope).await?.is_none());
                    assert!(entry(&next, "unrelated")?.load(&scope).await?.is_none());
                    drop(next);
                    assert!(!temporary.path().join(CacheRootLease::ACTIVE_FILE).exists());
                }
                Ok(())
            })
            .await
    }
}
