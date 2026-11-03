#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::StoredArtifact;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct NumberOperation;
struct DoubledNumberOperation;

fn key(name: &str) -> Result<CacheKey> {
    Ok(CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "artifact-binding", name])?,
        valid_for: Duration::from_secs(3600),
    })
}

fn executor(scope: CancellationScope) -> Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(AnonymousAuthenticationProvider),
        scope,
    ))
}

async fn retain(
    entry: &ArtifactEntry,
    value: u64,
    observation: ArtifactObservation,
    scope: &CancellationScope,
) -> Result<StoredArtifact> {
    entry
        .begin_attempt(BTreeMap::new(), scope)
        .await?
        .expect("full retention admits a raw attempt")
        .finish(
            ArtifactSet {
                payload: value.to_string().into_bytes(),
                diagnostics: BTreeMap::new(),
            },
            observation,
            scope,
        )
        .await
}

fn number<'a>(
    executor: &'a CloudTerrastodonExecutor,
    executions: &'a AtomicUsize,
    decodes: &'a AtomicUsize,
) -> Result<ExecutionFuture<'a, Arc<u64>>> {
    executor.cached_work_with_artifacts(
        ExecutorKind::Command,
        key("number")?,
        CacheOperationOwner::of::<NumberOperation>(),
        &AuthenticationSelection::Anonymous,
        NonEmptyString::new("synthetic-number-input")?,
        WorkReuse::ReusableRead,
        ArtifactRetention::Full,
        move |child, entry| async move {
            if let Some(loaded) = entry.load(child.cancellation()).await? {
                decodes.fetch_add(1, Ordering::SeqCst);
                let value = std::str::from_utf8(&loaded.artifacts.payload)?.parse::<u64>()?;
                return Ok(loaded.decoded(value));
            }
            let value = 41 + executions.fetch_add(1, Ordering::SeqCst) as u64;
            let observation = ArtifactObservation::now();
            let stored = retain(&entry, value, observation, child.cancellation()).await?;
            decodes.fetch_add(1, Ordering::SeqCst);
            assert!(entry.publish(&stored, child.cancellation()).await?);
            Ok(WorkOutput::new(value, observation))
        },
    )
}

fn doubled_number<'a>(
    executor: &'a CloudTerrastodonExecutor,
    executions: &'a AtomicUsize,
    decodes: &'a AtomicUsize,
    compositions: &'a AtomicUsize,
) -> Result<ExecutionFuture<'a, Arc<u64>>> {
    executor.cached_work_with_artifacts(
        ExecutorKind::InProcess,
        key("doubled-number")?,
        CacheOperationOwner::of::<DoubledNumberOperation>(),
        &AuthenticationSelection::Anonymous,
        NonEmptyString::new("synthetic-composition-input")?,
        WorkReuse::ReusableRead,
        ArtifactRetention::Full,
        move |child, entry| async move {
            // A previous aggregate attempt must never become a durable leaf hit.
            assert!(entry.load(child.cancellation()).await?.is_none());
            compositions.fetch_add(1, Ordering::SeqCst);
            let value = *number(&child, executions, decodes)?.await? * 2;
            let observation = ArtifactObservation::now();
            let stored = retain(&entry, value, observation, child.cancellation()).await?;
            assert!(stored.directory().exists());
            assert!(!entry.publish(&stored, child.cancellation()).await?);
            Ok(WorkOutput::new(value, observation))
        },
    )
}

#[tokio::test]
async fn memory_hits_skip_decoding_and_disk_imports_rejoin_shared_results() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = executor(scope)?;
            let executions = AtomicUsize::new(0);
            let decodes = AtomicUsize::new(0);
            let first = number(&executor, &executions, &decodes)?.await?;
            let hit = number(&executor, &executions, &decodes)?.await?;
            assert!(Arc::ptr_eq(&first, &hit));
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(decodes.load(Ordering::SeqCst), 1);

            executor.cache()?.release_memory()?;
            let restored = number(&executor, &executions, &decodes)?.await?;
            assert_eq!(*restored, 41);
            assert!(!Arc::ptr_eq(&first, &restored));
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(decodes.load(Ordering::SeqCst), 2);
            assert!(Arc::ptr_eq(
                &restored,
                &number(&executor, &executions, &decodes)?.await?
            ));
            assert_eq!(decodes.load(Ordering::SeqCst), 2);

            // The imported value participates in normal generation invalidation.
            executor
                .cache()?
                .invalidate(&key("number")?.scope, executor.cancellation())
                .await?;
            let refreshed = number(&executor, &executions, &decodes)?.await?;
            assert_eq!(*refreshed, 42);
            assert_eq!(*restored, 41);
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            assert_eq!(decodes.load(Ordering::SeqCst), 3);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn invalidation_while_decoding_a_disk_hit_prevents_republishing_it() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = executor(scope)?;
            let executions = AtomicUsize::new(0);
            let decodes = AtomicUsize::new(0);
            assert_eq!(*number(&executor, &executions, &decodes)?.await?, 41);
            executor.cache()?.release_memory()?;
            let (loaded, loading_finished) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let decode = executor.cached_work_with_artifacts(
                ExecutorKind::Command,
                key("number")?,
                CacheOperationOwner::of::<NumberOperation>(),
                &AuthenticationSelection::Anonymous,
                NonEmptyString::new("synthetic-number-input")?,
                WorkReuse::ReusableRead,
                ArtifactRetention::Full,
                |child, entry| async move {
                    let artifact = entry
                        .load(child.cancellation())
                        .await?
                        .expect("the seeded leaf has a disk selection");
                    loaded.send(()).expect("the invalidator is waiting");
                    released.await?;
                    let value = std::str::from_utf8(&artifact.artifacts.payload)?.parse::<u64>()?;
                    Ok(artifact.decoded(value))
                },
            )?;
            let invalidate = async {
                loading_finished.await?;
                executor
                    .cache()?
                    .invalidate(&key("number")?.scope, executor.cancellation())
                    .await?;
                release.send(()).expect("the decoder is waiting");
                Ok::<_, eyre::Report>(())
            };
            let (decoded, ()) = timeout(DEADLOCK_GUARD, async {
                tokio::try_join!(decode, invalidate)
            })
            .await??;
            // Its caller can use the observed result, but another call must run.
            assert_eq!(*decoded, 41);
            assert_eq!(*number(&executor, &executions, &decodes)?.await?, 42);
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn semantic_children_prevent_aggregate_disk_selection_and_invalidate_memory() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = executor(scope)?;
            let executions = AtomicUsize::new(0);
            let decodes = AtomicUsize::new(0);
            let compositions = AtomicUsize::new(0);
            let first = doubled_number(&executor, &executions, &decodes, &compositions)?.await?;
            assert_eq!(*first, 82);
            let hit = doubled_number(&executor, &executions, &decodes, &compositions)?.await?;
            assert!(Arc::ptr_eq(&first, &hit));
            assert_eq!(compositions.load(Ordering::SeqCst), 1);

            executor.cache()?.release_memory()?;
            let recomposed =
                doubled_number(&executor, &executions, &decodes, &compositions)?.await?;
            assert_eq!(*recomposed, 82);
            assert_eq!(compositions.load(Ordering::SeqCst), 2);
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(decodes.load(Ordering::SeqCst), 2);

            executor
                .cache()?
                .invalidate(&key("number")?.scope, executor.cancellation())
                .await?;
            let refreshed =
                doubled_number(&executor, &executions, &decodes, &compositions)?.await?;
            assert_eq!(*refreshed, 84);
            assert_eq!(*recomposed, 82);
            assert_eq!(compositions.load(Ordering::SeqCst), 3);
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}
