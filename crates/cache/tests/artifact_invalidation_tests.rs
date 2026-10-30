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
use cloud_terrastodon_cache::StoredArtifact;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

struct SyntheticRead;

fn identity(kind: ExecutorKind, components: &[&str], variant: &str) -> CacheEntryIdentity {
    CacheEntryIdentity {
        kind,
        scope: CacheScope::try_from_components(components.iter().copied()).unwrap(),
        auth_partition: AuthPartition::new("synthetic-session").unwrap(),
        fingerprint: NonEmptyString::new(variant).unwrap(),
    }
}

fn entry(cache: &WorkCache, identity: &CacheEntryIdentity) -> eyre::Result<ArtifactEntry> {
    cache.artifacts::<String>(
        identity.kind.clone(),
        CacheKey {
            scope: identity.scope.clone(),
            valid_for: Duration::from_secs(3600),
        },
        CacheOperationOwner::of::<SyntheticRead>(),
        identity.auth_partition.clone(),
        identity.fingerprint.clone(),
        ArtifactRetention::Full,
    )
}

async fn finish(
    entry: &ArtifactEntry,
    cancellation: &CancellationScope,
) -> eyre::Result<StoredArtifact> {
    entry
        .begin_attempt(BTreeMap::new(), cancellation)
        .await?
        .unwrap()
        .finish(
            ArtifactSet {
                payload: b"synthetic result".to_vec(),
                diagnostics: BTreeMap::new(),
            },
            ArtifactObservation::now(),
            cancellation,
        )
        .await
}

#[tokio::test]
async fn prefix_invalidation_finds_disk_only_entries_across_namespaces_and_variants()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let original = WorkCache::new(temporary.path())?;
            let identities = [
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected", "pages", "1"],
                    "first",
                ),
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected", "pages", "1"],
                    "second",
                ),
                identity(
                    ExecutorKind::Command,
                    &["projects", "selected", "pages", "2"],
                    "first",
                ),
                identity(
                    ExecutorKind::Other("synthetic".into()),
                    &["projects", "selected", "list"],
                    "first",
                ),
                identity(
                    ExecutorKind::Rest,
                    &["projects", "unrelated", "pages", "1"],
                    "first",
                ),
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected-suffix", "list"],
                    "first",
                ),
            ];
            let mut retained = Vec::new();
            for identity in &identities {
                let entry = entry(&original, identity)?;
                let stored = finish(&entry, &scope).await?;
                assert!(entry.publish(&stored, &scope).await?);
                retained.push(stored.directory().to_owned());
            }
            drop(original);
            // This owner has not claimed or loaded any of the persisted entries.
            let cache = WorkCache::new(temporary.path())?;
            cache
                .invalidate(
                    &CacheScope::try_from_components(["projects", "selected"])?,
                    &scope,
                )
                .await?;
            for (index, identity) in identities.iter().enumerate() {
                assert_eq!(
                    entry(&cache, identity)?.load(&scope).await?.is_some(),
                    index >= 4
                );
                assert!(retained[index].join("manifest.json").is_file());
                assert!(retained[index].join("payload.bin").is_file());
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn invalidation_prevents_an_inflight_attempt_from_selecting_its_later_outcome()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identity = identity(
                ExecutorKind::Rest,
                &["projects", "selected", "pages", "1"],
                "query",
            );
            let entry = entry(&cache, &identity)?;
            let attempt = entry.begin_attempt(BTreeMap::new(), &scope).await?.unwrap();
            cache
                .invalidate(
                    &CacheScope::try_from_components(["projects", "selected"])?,
                    &scope,
                )
                .await?;
            let stored = attempt
                .finish(
                    ArtifactSet {
                        payload: b"old-generation result".to_vec(),
                        diagnostics: BTreeMap::new(),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert!(!entry.publish(&stored, &scope).await?);
            assert!(entry.load(&scope).await?.is_none());
            assert!(stored.directory().join("payload.bin").is_file());
            let fresh = finish(&entry, &scope).await?;
            assert!(entry.publish(&fresh, &scope).await?);
            assert!(entry.load(&scope).await?.is_some());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn exact_rejection_preserves_independent_pages_and_request_variants() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identities = [
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected", "pages", "1"],
                    "first",
                ),
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected", "pages", "1"],
                    "second",
                ),
                identity(
                    ExecutorKind::Rest,
                    &["projects", "selected", "pages", "2"],
                    "first",
                ),
            ];
            let mut retained = Vec::new();
            for identity in &identities {
                let entry = entry(&cache, identity)?;
                let stored = finish(&entry, &scope).await?;
                assert!(entry.publish(&stored, &scope).await?);
                retained.push(stored.directory().to_owned());
            }
            cache.reject(&identities[0], &scope).await?;
            for (index, identity) in identities.iter().enumerate() {
                assert_eq!(
                    entry(&cache, identity)?.load(&scope).await?.is_some(),
                    index != 0
                );
                assert!(retained[index].join("payload.bin").is_file());
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn failed_prefix_invalidation_blocks_old_disk_and_exact_success_cannot_clear_it()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identity = identity(
                ExecutorKind::Rest,
                &["projects", "selected", "pages", "1"],
                "first",
            );
            let entry = entry(&cache, &identity)?;
            let stored = finish(&entry, &scope).await?;
            assert!(entry.publish(&stored, &scope).await?);
            let selection = entry.directory()?.join("selection.json");
            let previous = entry.directory()?.join("previous-selection.json");
            fs::rename(&selection, &previous)?;
            fs::create_dir(&selection)?;
            let prefix = CacheScope::try_from_components(["projects", "selected"])?;
            assert!(cache.invalidate(&prefix, &scope).await.is_err());
            fs::remove_dir(&selection)?;
            fs::rename(&previous, &selection)?;
            cache.release_memory()?;
            assert!(entry.load(&scope).await?.is_none());
            let quarantined = finish(&entry, &scope).await?;
            assert!(!entry.publish(&quarantined, &scope).await?);
            cache.reject(&identity, &scope).await?;
            let exact_success = finish(&entry, &scope).await?;
            assert!(!entry.publish(&exact_success, &scope).await?);
            assert!(entry.load(&scope).await?.is_none());
            cache.invalidate(&prefix, &scope).await?;
            let recovered = finish(&entry, &scope).await?;
            assert!(entry.publish(&recovered, &scope).await?);
            assert!(entry.load(&scope).await?.is_some());
            assert!(stored.directory().join("payload.bin").is_file());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_bound_leaf_with_semantic_children_cannot_publish_aggregate_artifacts() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let parent = identity(ExecutorKind::Rest, &["synthetic", "aggregate"], "query");
            let child = identity(ExecutorKind::InProcess, &["synthetic", "child"], "query");
            let artifact = entry(&cache, &parent)?;
            let cache = &cache;
            let artifact = &artifact;
            let value = cache
                .cached_work(
                    parent,
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    Duration::from_secs(3600),
                    None,
                    &scope,
                    |attempt, child_scope| async move {
                        let attempt = attempt.unwrap();
                        cache
                            .cached_work(
                                child,
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                Duration::from_secs(3600),
                                Some(&attempt),
                                &child_scope,
                                |_, _| async {
                                    Ok(WorkOutput::new(
                                        String::from("child"),
                                        ArtifactObservation::now(),
                                    ))
                                },
                            )?
                            .await?;
                        let bound = artifact.clone().for_work(&attempt)?;
                        let stored = finish(&bound, &child_scope).await?;
                        assert!(!bound.publish(&stored, &child_scope).await?);
                        Ok(WorkOutput::new(
                            String::from("aggregate"),
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await?;
            assert_eq!(&*value, "aggregate");
            assert!(artifact.load(&scope).await?.is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn binding_artifacts_after_invalidation_keeps_the_work_attempts_original_generation()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identity = identity(ExecutorKind::Rest, &["synthetic", "late-artifact"], "query");
            let artifact = entry(&cache, &identity)?;
            let cache = &cache;
            let artifact = &artifact;
            let value = cache
                .cached_work(
                    identity.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    Duration::from_secs(3600),
                    None,
                    &scope,
                    |attempt, child_scope| async move {
                        cache.invalidate(&identity.scope, &child_scope).await?;
                        let bound = artifact.clone().for_work(&attempt.unwrap())?;
                        let stored = finish(&bound, &child_scope).await?;
                        assert!(!bound.publish(&stored, &child_scope).await?);
                        Ok(WorkOutput::new(
                            String::from("original caller result"),
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await?;
            assert_eq!(&*value, "original caller result");
            assert!(artifact.load(&scope).await?.is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_child_rejected_after_leaf_commit_prevents_both_memory_and_disk_reuse() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let parent = identity(ExecutorKind::Rest, &["synthetic", "frozen-leaf"], "query");
            let child = identity(
                ExecutorKind::InProcess,
                &["synthetic", "late-child"],
                "query",
            );
            let artifact = entry(&cache, &parent)?;
            let executions = AtomicUsize::new(0);
            let cache = &cache;
            let artifact = &artifact;
            let child = &child;
            let executions = &executions;
            for _ in 0..2 {
                cache
                    .cached_work(
                        parent.clone(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        WorkReuse::ReusableRead,
                        Duration::from_secs(3600),
                        None,
                        &scope,
                        |attempt, child_scope| async move {
                            executions.fetch_add(1, Ordering::SeqCst);
                            let attempt = attempt.unwrap();
                            let bound = artifact.clone().for_work(&attempt)?;
                            let stored = finish(&bound, &child_scope).await?;
                            assert!(bound.publish(&stored, &child_scope).await?);
                            let rejected = cache.cached_work(
                                child.clone(),
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                Duration::from_secs(3600),
                                Some(&attempt),
                                &child_scope,
                                |_, _| async {
                                    Err::<WorkOutput<String>, _>(eyre::eyre!(
                                        "late child cannot execute"
                                    ))
                                },
                            );
                            assert!(rejected.is_err());
                            Ok(WorkOutput::new(
                                String::from("caught late-child failure"),
                                ArtifactObservation::now(),
                            ))
                        },
                    )?
                    .await?;
                assert!(artifact.load(&scope).await?.is_none());
            }
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn failure_after_a_bound_leaf_commit_keeps_the_selection_quarantined() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identity = identity(ExecutorKind::Rest, &["synthetic", "failed-leaf"], "query");
            let artifact = entry(&cache, &identity)?;
            let artifact = &artifact;
            let outcome = cache
                .cached_work(
                    identity,
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    Duration::from_secs(3600),
                    None,
                    &scope,
                    |attempt, child_scope| async move {
                        let bound = artifact.clone().for_work(&attempt.unwrap())?;
                        let stored = finish(&bound, &child_scope).await?;
                        assert!(bound.publish(&stored, &child_scope).await?);
                        Err::<WorkOutput<String>, _>(eyre::eyre!(
                            "canonical callback failed after commit"
                        ))
                    },
                )?
                .await;
            assert!(
                outcome
                    .unwrap_err()
                    .to_string()
                    .contains("canonical callback failed")
            );
            cache.release_memory()?;
            assert!(artifact.load(&scope).await?.is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_loaded_proof_cannot_outlive_its_successful_or_failed_initializer() -> eyre::Result<()> {
    CancellationToken::new().with_async_scope(|scope| async move {
        for fail_original in [false, true] {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let identity = identity(ExecutorKind::Rest, &["synthetic", "bound-proof"], "query");
            let artifact = entry(&cache, &identity)?;
            let retained = Mutex::new(None);
            let retained = &retained;
            let artifact = &artifact;
            let first = cache.cached_work(identity.clone(), CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead, Duration::from_secs(3600), None, &scope,
                |attempt, child_scope| async move {
                    let bound = artifact.clone().for_work(&attempt.unwrap())?;
                    let stored = finish(&bound, &child_scope).await?;
                    assert!(bound.publish(&stored, &child_scope).await?);
                    let loaded = bound.load(&child_scope).await?;
                    *retained.lock().unwrap() = loaded;
                    if fail_original {
                        return Err(eyre::eyre!("original initializer failed after loading"));
                    }
                    Ok(WorkOutput::new(String::from("original result"), ArtifactObservation::now()))
                })?.await;
            assert_eq!(first.is_err(), fail_original);
            cache.release_memory()?;
            let loaded = retained.lock().unwrap().take().unwrap();
            let imported = cache.cached_work(identity, CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead, Duration::from_secs(3600), None, &scope,
                |_, _| async move { Ok(loaded.decoded(String::from("old decoded result"))) })?.await;
            assert!(imported.unwrap_err().to_string().contains("initializer"));
        }
        Ok(())
    }).await
}
