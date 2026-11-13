#![cfg(test)]

use cloud_terrastodon_cache::ArtifactEntry;
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
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::collections::BTreeMap;
use std::fs;
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::Barrier;
use tokio::time::timeout;

const DEADLOCK_GUARD: Duration = Duration::from_secs(45);
const BLOCKED_OBSERVATION: Duration = Duration::from_millis(200);
const VALID_FOR: Duration = Duration::from_secs(3600);

struct SyntheticRead;

fn identity(name: &str) -> CacheEntryIdentity {
    CacheEntryIdentity {
        kind: ExecutorKind::Rest,
        scope: CacheScope::try_from_components(["synthetic", name]).unwrap(),
        auth_partition: AuthPartition::new("synthetic-session").unwrap(),
        fingerprint: NonEmptyString::new("synthetic-request").unwrap(),
    }
}

fn entry(cache: &WorkCache) -> Result<ArtifactEntry> {
    let identity = identity("shared");
    cache.artifacts::<String>(
        identity.kind,
        CacheKey {
            scope: identity.scope,
            valid_for: VALID_FOR,
        },
        CacheOperationOwner::of::<SyntheticRead>(),
        identity.auth_partition,
        identity.fingerprint,
        ArtifactRetention::Full,
    )
}

async fn read(
    cache: &WorkCache,
    signals: &Path,
    fresh_value: &str,
    scope: &CancellationScope,
) -> Result<Arc<String>> {
    cache
        .cached_work(
            identity("shared"),
            CacheOperationOwner::of::<SyntheticRead>(),
            WorkReuse::ReusableRead,
            VALID_FOR,
            None,
            scope,
            async |attempt, child| {
                let artifact = entry(cache)?.for_work(&attempt.unwrap())?;
                if let Some(loaded) = artifact.load(&child).await? {
                    let value = std::str::from_utf8(&loaded.artifacts.payload)?.to_owned();
                    return Ok(loaded.decoded(value));
                }
                // This stands in for transport execution; disk decoding does not
                // increment it. Both processes write it only while owning the cache.
                writeln!(
                    fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(signals.join("executions"))?,
                    "{fresh_value}"
                )?;
                let observation = ArtifactObservation::now();
                let stored = artifact
                    .begin_attempt(BTreeMap::new(), &child)
                    .await?
                    .unwrap()
                    .finish(
                        ArtifactSet {
                            payload: fresh_value.as_bytes().to_vec(),
                            diagnostics: BTreeMap::new(),
                        },
                        observation,
                        &child,
                    )
                    .await?;
                assert!(artifact.publish(&stored, &child).await?);
                Ok(WorkOutput::new(fresh_value.to_owned(), observation))
            },
        )?
        .await
}

// Every process has an RAII guard, including failed/timeout assertions, so a
// broken locking implementation cannot leave a detached child holding the root.
struct TestProcess {
    child: Child,
    signals: PathBuf,
    label: String,
    log: PathBuf,
}

impl TestProcess {
    fn spawn(signals: &Path, role: &str, label: &str) -> Result<Self> {
        let log = signals.join(format!("{label}.log"));
        let output = fs::File::create(&log)?;
        let child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "ownership_process_child",
                "--ignored",
                "--nocapture",
            ])
            .env("CT_CACHE_OWNERSHIP_TEST_ROLE", role)
            .env("CT_CACHE_OWNERSHIP_TEST_LABEL", label)
            .env("CT_CACHE_OWNERSHIP_TEST_SIGNALS", signals)
            .stdin(Stdio::null())
            .stdout(output.try_clone()?)
            .stderr(output)
            .spawn()?;
        Ok(Self {
            child,
            signals: signals.to_path_buf(),
            label: label.to_owned(),
            log,
        })
    }

    fn signal(&self, name: &str) -> Result<()> {
        signal(&self.signals, &self.label, name)
    }

    fn wait_for(&mut self, name: &str) -> Result<()> {
        let marker = self.signals.join(format!("{}.{name}", self.label));
        let deadline = Instant::now() + DEADLOCK_GUARD;
        while !marker.exists() {
            if let Some(status) = self.child.try_wait()? {
                bail!(
                    "child {} exited {status} before {name}: {}",
                    self.label,
                    fs::read_to_string(&self.log)?
                );
            }
            ensure!(
                Instant::now() < deadline,
                "child {} timed out waiting for {name}",
                self.label
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        let deadline = Instant::now() + DEADLOCK_GUARD;
        loop {
            if let Some(status) = self.child.try_wait()? {
                ensure!(
                    status.success(),
                    "child {} failed: {}",
                    self.label,
                    fs::read_to_string(&self.log)?
                );
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "child {} timed out exiting",
                self.label
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for TestProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn signal(signals: &Path, label: &str, name: &str) -> Result<()> {
    fs::write(signals.join(format!("{label}.{name}")), [])?;
    Ok(())
}

async fn wait_for_signal(signals: &Path, label: &str, name: &str) -> Result<()> {
    timeout(DEADLOCK_GUARD, async {
        while !signals.join(format!("{label}.{name}")).exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await?;
    Ok(())
}

async fn blocked_then_complete<T>(
    future: impl Future<Output = Result<T>>,
    signals: &Path,
    label: &str,
) -> Result<T> {
    tokio::pin!(future);
    // A single Pending poll could just be a filesystem worker starting. Poll
    // through a bounded observation interval while the other process holds the
    // lock, then tell the parent that this operation really is still waiting.
    assert!(timeout(BLOCKED_OBSERVATION, &mut future).await.is_err());
    signal(signals, label, "waiting")?;
    timeout(DEADLOCK_GUARD, future).await?
}

#[test]
#[ignore = "subprocess entry point; launched only by the isolated ownership tests"]
fn ownership_process_child() -> Result<()> {
    let role = std::env::var("CT_CACHE_OWNERSHIP_TEST_ROLE")?;
    let label = std::env::var("CT_CACHE_OWNERSHIP_TEST_LABEL")?;
    let signals = PathBuf::from(std::env::var_os("CT_CACHE_OWNERSHIP_TEST_SIGNALS").unwrap());
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let cache = WorkCache::new(signals.join("cache"))?;
            let token = CancellationToken::new();
            token
                .with_async_scope(async |scope| {
                    match role.as_str() {
                        "owner" | "crash" => {
                            assert_eq!(
                                read(&cache, &signals, "original", &scope).await?.as_str(),
                                "original"
                            );
                            let unfinished = if role == "crash" {
                                // Leave an incomplete attempt next to a previously
                                // selected result. Neither may survive as reusable
                                // evidence after this owner is killed abruptly.
                                entry(&cache)?
                                    .begin_attempt(BTreeMap::new(), &scope)
                                    .await?
                            } else {
                                None
                            };
                            signal(&signals, &label, "ready")?;
                            wait_for_signal(&signals, &label, "release").await?;
                            drop(unfinished);
                        }
                        "read" => {
                            let value = blocked_then_complete(
                                read(&cache, &signals, "replacement", &scope),
                                &signals,
                                &label,
                            )
                            .await?;
                            assert_eq!(value.as_str(), "original");
                        }
                        "load" => {
                            let artifact = entry(&cache)?;
                            let loaded =
                                blocked_then_complete(artifact.load(&scope), &signals, &label)
                                    .await?
                                    .unwrap();
                            assert_eq!(loaded.artifacts.payload, b"original");
                        }
                        "inventory" => {
                            assert_eq!(
                                blocked_then_complete(cache.inventory(&scope), &signals, &label)
                                    .await?
                                    .len(),
                                1
                            );
                        }
                        "invalidate" => {
                            blocked_then_complete(
                                cache.invalidate(&identity("shared").scope, &scope),
                                &signals,
                                &label,
                            )
                            .await?;
                            assert!(entry(&cache)?.load(&scope).await?.is_none());
                        }
                        "cancel" => {
                            let waiting = cache.inventory(&scope);
                            tokio::pin!(waiting);
                            assert!(timeout(BLOCKED_OBSERVATION, &mut waiting).await.is_err());
                            signal(&signals, &label, "waiting")?;
                            wait_for_signal(&signals, &label, "cancel").await?;
                            token.request_cancel("cancel ownership waiter");
                            assert!(timeout(DEADLOCK_GUARD, waiting).await?.is_err());
                            signal(&signals, &label, "cancelled")?;
                            // Keep this handle alive while a third process takes
                            // the root. A detached blocking lock acquisition would
                            // otherwise acquire it later and prevent that handoff.
                            wait_for_signal(&signals, &label, "release").await?;
                        }
                        "reopen" | "recover" => {
                            let value = read(&cache, &signals, "replacement", &scope).await?;
                            assert_eq!(
                                value.as_str(),
                                if role == "recover" {
                                    "replacement"
                                } else {
                                    "original"
                                }
                            );
                        }
                        _ => {
                            bail!("unknown synthetic subprocess role {role}");
                        }
                    }
                    Ok(())
                })
                .await
        })
}

#[test]
fn separate_processes_wait_and_recheck_disk_after_a_clean_handoff() -> Result<()> {
    for operation in ["read", "load", "inventory", "invalidate"] {
        let temporary = tempfile::tempdir()?;
        let mut owner = TestProcess::spawn(temporary.path(), "owner", "owner")?;
        owner.wait_for("ready")?;
        let mut follower = TestProcess::spawn(temporary.path(), operation, "follower")?;
        follower.wait_for("waiting")?;
        assert!(owner.child.try_wait()?.is_none());
        owner.signal("release")?;
        owner.finish()?;
        follower.finish()?;
        assert_eq!(
            fs::read_to_string(temporary.path().join("executions"))?,
            "original\n"
        );
    }
    Ok(())
}

#[test]
fn a_cancelled_process_waiter_cannot_acquire_the_root_later() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut owner = TestProcess::spawn(temporary.path(), "owner", "owner")?;
    owner.wait_for("ready")?;
    let mut cancelled = TestProcess::spawn(temporary.path(), "cancel", "cancelled")?;
    cancelled.wait_for("waiting")?;
    cancelled.signal("cancel")?;
    cancelled.wait_for("cancelled")?;
    owner.signal("release")?;
    owner.finish()?;
    let mut next = TestProcess::spawn(temporary.path(), "reopen", "next")?;
    next.finish()?;
    assert!(cancelled.child.try_wait()?.is_none());
    cancelled.signal("release")?;
    cancelled.finish()?;
    assert_eq!(
        fs::read_to_string(temporary.path().join("executions"))?,
        "original\n"
    );
    Ok(())
}

#[test]
fn a_killed_owner_releases_the_os_lock_and_recovery_rejects_old_selections() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let mut owner = TestProcess::spawn(temporary.path(), "crash", "owner")?;
    owner.wait_for("ready")?;
    owner.child.kill()?;
    assert!(!owner.child.wait()?.success());
    let mut recovered = TestProcess::spawn(temporary.path(), "recover", "recovered")?;
    recovered.finish()?;
    assert_eq!(
        fs::read_to_string(temporary.path().join("executions"))?,
        "original\nreplacement\n"
    );
    // Recovery invalidates selections, preserving both complete and interrupted
    // attempts for inspection instead of deleting the evidence of the crash.
    let cache = WorkCache::new(temporary.path().join("cache"))?;
    assert_eq!(
        fs::read_dir(entry(&cache)?.directory()?.join("attempts"))?.count(),
        3
    );
    Ok(())
}

#[tokio::test]
async fn claims_are_lazy_and_artifact_handles_retain_ownership_after_cache_clones_drop()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let temporary = tempfile::tempdir()?;
            let root = temporary.path().join("cache");
            let cache = WorkCache::new(&root)?;
            let clone = cache.clone();
            let artifact = entry(&cache)?;
            let _ = artifact.directory()?;
            let work = cache.cached_work(
                identity("unstarted"),
                CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                None,
                &scope,
                async |_, _| eyre::Ok(WorkOutput::new((), ArtifactObservation::now())),
            )?;
            drop(work);
            assert!(
                !root.exists(),
                "construction and claiming must perform no filesystem I/O"
            );
            assert!(cache.inventory(&scope).await?.is_empty());
            drop(cache);
            let next = WorkCache::new(&root)?;
            let pending = next.inventory(&scope);
            tokio::pin!(pending);
            assert!(timeout(BLOCKED_OBSERVATION, &mut pending).await.is_err());
            drop(clone);
            assert!(timeout(BLOCKED_OBSERVATION, &mut pending).await.is_err());
            drop(artifact);
            assert!(timeout(DEADLOCK_GUARD, pending).await??.is_empty());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn dropped_acquisition_does_not_poison_shared_initialization() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let temporary = tempfile::tempdir()?;
            let root = temporary.path().join("shared");
            let owner = WorkCache::new(&root)?;
            owner.acquire(&scope).await?;
            let waiting = WorkCache::new(&root)?;
            let clone = waiting.clone();
            let mut first = Box::pin(waiting.acquire(&scope));
            assert!(timeout(BLOCKED_OBSERVATION, &mut first).await.is_err());
            let mut second = Box::pin(clone.acquire(&scope));
            assert!(timeout(BLOCKED_OBSERVATION, &mut second).await.is_err());

            // Contention concerns only this root, even when acquisition itself
            // is still initializing. An unrelated root remains usable now.
            let unrelated = WorkCache::new(temporary.path().join("unrelated"))?;
            timeout(DEADLOCK_GUARD, unrelated.acquire(&scope)).await??;

            // Drop the initializer currently holding the shared initialization
            // permit. Another clone must retry rather than inheriting a stuck
            // OnceCell or depending on the abandoned future to finish.
            drop(first);
            drop(owner);
            timeout(DEADLOCK_GUARD, second).await??;
            timeout(DEADLOCK_GUARD, waiting.acquire(&scope)).await??;
            Ok(())
        })
        .await
}

#[tokio::test]
async fn shared_owners_keep_different_initializers_concurrent() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            let clone = cache.clone();
            let admitted = Barrier::new(2);
            let first = cache.cached_work(
                identity("first"),
                CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                None,
                &scope,
                async |_, _| {
                    admitted.wait().await;
                    eyre::Ok(WorkOutput::new(1_u64, ArtifactObservation::now()))
                },
            )?;
            let second = clone.cached_work(
                identity("second"),
                CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                None,
                &scope,
                async |_, _| {
                    admitted.wait().await;
                    eyre::Ok(WorkOutput::new(2_u64, ArtifactObservation::now()))
                },
            )?;
            let (first, second) = timeout(DEADLOCK_GUARD, async {
                tokio::join!(async { first.await }, async { second.await })
            })
            .await?;
            assert_eq!(*first?, 1);
            assert_eq!(*second?, 2);
            Ok(())
        })
        .await
}
