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
use std::collections::BTreeMap;
use std::fs;
use std::future::IntoFuture;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const VALID_FOR: Duration = Duration::from_secs(3600);
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct SyntheticRead;

fn identity() -> CacheEntryIdentity {
    CacheEntryIdentity {
        kind: ExecutorKind::Rest,
        scope: CacheScope::try_from_components(["synthetic", "projects", "selected"]).unwrap(),
        auth_partition: AuthPartition::new("synthetic-session").unwrap(),
        fingerprint: NonEmptyString::new("synthetic-query").unwrap(),
    }
}

fn entry(cache: &WorkCache) -> eyre::Result<ArtifactEntry> {
    let identity = identity();
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

async fn seed(entry: &ArtifactEntry, cancellation: &CancellationScope) -> eyre::Result<()> {
    let stored = entry
        .begin_attempt(BTreeMap::new(), cancellation)
        .await?
        .unwrap()
        .finish(
            ArtifactSet {
                payload: b"old synthetic snapshot".to_vec(),
                diagnostics: BTreeMap::new(),
            },
            ArtifactObservation::now(),
            cancellation,
        )
        .await?;
    assert!(entry.publish(&stored, cancellation).await?);
    Ok(())
}

async fn read(
    cache: &WorkCache,
    scope: &CancellationScope,
    executions: &AtomicUsize,
    fresh_value: &str,
) -> eyre::Result<Arc<String>> {
    cache
        .cached_work(
            identity(),
            CacheOperationOwner::of::<SyntheticRead>(),
            WorkReuse::ReusableRead,
            VALID_FOR,
            None,
            scope,
            |attempt, child| async move {
                let artifact = entry(cache)?.for_work(&attempt.unwrap())?;
                if let Some(loaded) = artifact.load(&child).await? {
                    let value = std::str::from_utf8(&loaded.artifacts.payload)?.to_owned();
                    return Ok(loaded.decoded(value));
                }
                executions.fetch_add(1, Ordering::SeqCst);
                Ok(WorkOutput::new(
                    fresh_value.to_owned(),
                    ArtifactObservation::now(),
                ))
            },
        )?
        .await
}

#[tokio::test]
async fn invalidation_during_disk_decoding_returns_the_snapshot_without_publishing_it()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            seed(&entry(&cache)?, &scope).await?;
            let (loaded_sender, loaded_receiver) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let decoding = cache
                .cached_work(
                    identity(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |attempt, child| {
                        let cache = &cache;
                        async move {
                            let artifact = entry(cache)?.for_work(&attempt.unwrap())?;
                            let loaded = artifact.load(&child).await?.unwrap();
                            loaded_sender.send(()).unwrap();
                            released.await?;
                            let value = std::str::from_utf8(&loaded.artifacts.payload)?.to_owned();
                            Ok(loaded.decoded(value))
                        }
                    },
                )?
                .into_future();
            tokio::pin!(decoding);
            tokio::select! {
                ready = timeout(DEADLOCK_GUARD, loaded_receiver) => ready??,
                unexpected = &mut decoding => panic!("decoder completed before release: {unexpected:?}"),
            }
            cache.invalidate(&identity().scope, &scope).await?;
            release.send(()).unwrap();
            let snapshot = timeout(DEADLOCK_GUARD, decoding).await??;
            assert_eq!(snapshot.as_str(), "old synthetic snapshot");

            let executions = AtomicUsize::new(0);
            let fresh = read(&cache, &scope, &executions, "new synthetic snapshot").await?;
            assert_eq!(fresh.as_str(), "new synthetic snapshot");
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert!(!Arc::ptr_eq(&snapshot, &fresh));
            assert!(Arc::ptr_eq(
                &fresh,
                &read(&cache, &scope, &executions, "must not execute").await?
            ));
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn failed_disk_invalidation_allows_fresh_memory_reuse_but_blocks_old_disk_reload()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let artifact = entry(&cache)?;
            seed(&artifact, &scope).await?;
            let executions = AtomicUsize::new(0);
            let original = read(&cache, &scope, &executions, "must not execute").await?;
            assert_eq!(original.as_str(), "old synthetic snapshot");
            assert_eq!(executions.load(Ordering::SeqCst), 0);

            // This fixture's root is a fresh temporary directory. Replacing its
            // selection with a directory makes durable removal fail safely.
            let selection = artifact.directory()?.join("selection.json");
            let previous = artifact.directory()?.join("previous-selection.json");
            fs::rename(&selection, &previous)?;
            fs::create_dir(&selection)?;
            assert!(cache.invalidate(&identity().scope, &scope).await.is_err());
            fs::remove_dir(&selection)?;
            fs::rename(&previous, &selection)?;
            assert!(artifact.load(&scope).await?.is_none());

            let fresh = read(&cache, &scope, &executions, "fresh despite disk failure").await?;
            assert_eq!(fresh.as_str(), "fresh despite disk failure");
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert!(Arc::ptr_eq(
                &fresh,
                &read(&cache, &scope, &executions, "must not execute").await?
            ));
            assert_eq!(executions.load(Ordering::SeqCst), 1);

            cache.release_memory()?;
            assert!(artifact.load(&scope).await?.is_none());
            let refilled = read(&cache, &scope, &executions, "fresh after eviction").await?;
            assert_eq!(refilled.as_str(), "fresh after eviction");
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            assert!(!Arc::ptr_eq(&fresh, &refilled));
            assert_eq!(original.as_str(), "old synthetic snapshot");
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_fresh_owner_rebuilds_aggregates_with_current_decoding_of_valid_raw_children()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let mut cache = WorkCache::new(temporary.path())?;
            seed(&entry(&cache)?, &scope).await?;
            let assemblies = AtomicUsize::new(0);
            let decodes = AtomicUsize::new(0);
            let executions = AtomicUsize::new(0);
            let aggregate = CacheEntryIdentity {
                kind: ExecutorKind::InProcess,
                scope: CacheScope::try_from_components(["synthetic", "aggregate"])?,
                ..identity()
            };
            for expected in [1, 2] {
                let value = cache
                    .cached_work(
                        aggregate.clone(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        WorkReuse::ReusableRead,
                        VALID_FOR,
                        None,
                        &scope,
                        |parent, child| {
                            let cache = &cache;
                            let assemblies = &assemblies;
                            let decodes = &decodes;
                            let executions = &executions;
                            async move {
                                let assembly = assemblies.fetch_add(1, Ordering::SeqCst) + 1;
                                let leaf = cache
                                    .cached_work(
                                        identity(),
                                        CacheOperationOwner::of::<SyntheticRead>(),
                                        WorkReuse::ReusableRead,
                                        VALID_FOR,
                                        parent.as_ref(),
                                        &child,
                                        |attempt, leaf_scope| async move {
                                            let artifact =
                                                entry(cache)?.for_work(&attempt.unwrap())?;
                                            let Some(loaded) = artifact.load(&leaf_scope).await?
                                            else {
                                                executions.fetch_add(1, Ordering::SeqCst);
                                                return Ok(WorkOutput::new(
                                                    "fresh execution was unexpectedly required"
                                                        .to_owned(),
                                                    ArtifactObservation::now(),
                                                ));
                                            };
                                            let decoder =
                                                decodes.fetch_add(1, Ordering::SeqCst) + 1;
                                            let value = format!(
                                                "decoder {decoder}: {}",
                                                std::str::from_utf8(&loaded.artifacts.payload)?
                                            );
                                            Ok(loaded.decoded(value))
                                        },
                                    )?
                                    .await?;
                                Ok(WorkOutput::new(
                                    format!("assembly {assembly}: {leaf}"),
                                    ArtifactObservation::now(),
                                ))
                            }
                        },
                    )?
                    .await?;
                assert_eq!(
                    value.as_str(),
                    format!("assembly {expected}: decoder {expected}: old synthetic snapshot")
                );
                let warm = cache
                    .cached_work::<String, _, _>(
                        aggregate.clone(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        WorkReuse::ReusableRead,
                        VALID_FOR,
                        None,
                        &scope,
                        |_, _| async { panic!("a warm aggregate must not rebuild") },
                    )?
                    .await?;
                assert!(Arc::ptr_eq(&value, &warm));
                assert_eq!(assemblies.load(Ordering::SeqCst), expected);
                assert_eq!(decodes.load(Ordering::SeqCst), expected);
                assert_eq!(executions.load(Ordering::SeqCst), 0);
                if expected == 1 {
                    // The replacement owner retains only the synthetic disk
                    // artifacts; neither aggregate nor decoded leaf is restored.
                    cache = WorkCache::new(temporary.path())?;
                }
            }
            Ok(())
        })
        .await
}
