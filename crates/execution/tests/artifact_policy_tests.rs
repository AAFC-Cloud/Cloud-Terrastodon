#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactFilePath;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use eyre::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::Barrier;
use tokio::time::timeout;

static AUTHENTICATION: std::sync::LazyLock<AuthenticationSelection> =
    std::sync::LazyLock::new(|| AuthenticationSelection::AzureDevOpsPersonalAccessToken {
        account_id: "11111111-1111-1111-1111-111111111111"
            .parse()
            .expect("synthetic account ID"),
    });
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct NumberOperation;
struct CompositionOperation;

#[derive(Default)]
struct CredentialArtifactProbe {
    initializations: AtomicUsize,
}

impl AuthenticationProvider for CredentialArtifactProbe {
    fn admit(&self, _selection: &AuthenticationSelection) -> Result<AuthPartition> {
        AuthPartition::new("synthetic-credential-session")
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            let work = executor.cached_work_with_artifacts(
                ExecutorKind::Command,
                key("credential-infrastructure"),
                CacheOperationOwner::of::<NumberOperation>(),
                selection,
                fingerprint(),
                WorkReuse::ReusableRead,
                ArtifactRetention::Full,
                |_, _| async {
                    self.initializations.fetch_add(1, Ordering::SeqCst);
                    Ok(WorkOutput::new(7_u64, ArtifactObservation::now()))
                },
            );
            assert!(
                work.err()
                    .expect("credential work must be rejected before artifact binding")
                    .to_string()
                    .contains("credential infrastructure")
            );
            Ok(Authorization::Anonymous)
        })
    }
}

fn key(name: &str) -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", name]).unwrap(),
        valid_for: Duration::from_secs(3600),
    }
}

fn fingerprint() -> NonEmptyString {
    NonEmptyString::new("synthetic-input").unwrap()
}

async fn publish_number(
    entry: &ArtifactEntry,
    value: u64,
    cancellation: &CancellationScope,
) -> Result<(bool, ArtifactObservation)> {
    let observation = ArtifactObservation::now();
    let stored = entry
        .begin_attempt(BTreeMap::new(), cancellation)
        .await?
        .expect("full retention must stage an artifact attempt")
        .finish(
            ArtifactSet {
                payload: value.to_le_bytes().to_vec(),
                diagnostics: BTreeMap::new(),
            },
            observation,
            cancellation,
        )
        .await?;
    Ok((entry.publish(&stored, cancellation).await?, observation))
}

#[tokio::test]
async fn execute_every_time_ignores_existing_disk_results_and_never_joins_or_selects() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let prior = cache.artifacts::<u64>(
                ExecutorKind::Command,
                key("always-execute"),
                CacheOperationOwner::of::<NumberOperation>(),
                authentication.admit(&AUTHENTICATION)?,
                fingerprint(),
                ArtifactRetention::Full,
            )?;
            assert!(publish_number(&prior, 7, &scope).await?.0);
            let executor = CloudTerrastodonExecutor::new(cache, authentication, scope);
            let starts = AtomicUsize::new(0);
            let simultaneous = Barrier::new(2);
            let execute = || {
                executor.cached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("always-execute"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    WorkReuse::ExecuteEveryTime,
                    ArtifactRetention::Full,
                    async |child, entry| {
                        assert!(entry.load(child.cancellation()).await?.is_none());
                        let number = starts.fetch_add(1, Ordering::SeqCst) as u64 + 10;
                        // Joining or serializing these executions would prevent
                        // either invocation from passing this shared barrier.
                        simultaneous.wait().await;
                        let (selected, observation) =
                            publish_number(&entry, number, child.cancellation()).await?;
                        assert!(!selected);
                        Ok(WorkOutput::new(number, observation))
                    },
                )
            };
            let first = execute()?;
            let second = execute()?;
            let (first, second) =
                timeout(DEADLOCK_GUARD, async { tokio::join!(first, second) }).await?;
            let first = first?;
            let second = second?;
            assert_ne!(*first, *second);
            assert!(!Arc::ptr_eq(&first, &second));
            assert_eq!(starts.load(Ordering::SeqCst), 2);
            // The nonreusable invocations retain diagnostics without replacing
            // the previously selected read artifact under the same identity.
            assert_eq!(
                prior
                    .load(executor.cancellation())
                    .await?
                    .unwrap()
                    .artifacts
                    .payload,
                7_u64.to_le_bytes()
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn disabled_artifact_retention_keeps_decoded_memory_reuse_without_writing_inputs()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let executor = CloudTerrastodonExecutor::new(
                cache,
                Arc::new(TestAuthenticationProvider::offline(
                    "synthetic-test-session",
                )?),
                scope,
            );
            let starts = AtomicUsize::new(0);
            let execute = || {
                executor.cached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("memory-only"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    WorkReuse::ReusableRead,
                    ArtifactRetention::Disabled,
                    async |child, entry| {
                        starts.fetch_add(1, Ordering::SeqCst);
                        assert!(entry.load(child.cancellation()).await?.is_none());
                        assert!(
                            entry
                                .begin_attempt(
                                    BTreeMap::from([(
                                        ArtifactFilePath::new("synthetic-input.txt")?,
                                        b"offline input that must not be persisted".to_vec(),
                                    )]),
                                    child.cancellation(),
                                )
                                .await?
                                .is_none()
                        );
                        Ok(WorkOutput::new(42_u64, ArtifactObservation::now()))
                    },
                )
            };
            let first = execute()?.await?;
            let second = execute()?.await?;
            assert!(Arc::ptr_eq(&first, &second));
            assert_eq!(starts.load(Ordering::SeqCst), 1);
            assert!(std::fs::read_dir(root)?.next().is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn credential_infrastructure_cannot_bind_artifacts_or_start_semantic_work() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let provider = Arc::new(CredentialArtifactProbe::default());
            let executor = CloudTerrastodonExecutor::new(cache, provider.clone(), scope);
            executor
                .authorization(&AUTHENTICATION, AzureCredentialResource::AzureDevOps)
                .await?;
            assert_eq!(provider.initializations.load(Ordering::SeqCst), 0);
            assert!(std::fs::read_dir(root)?.next().is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn authentication_change_rejects_memory_fill_and_quarantines_already_published_artifacts()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let original = cache.artifacts::<u64>(
                ExecutorKind::Command,
                key("changed-authentication"),
                CacheOperationOwner::of::<NumberOperation>(),
                authentication.admit(&AUTHENTICATION)?,
                fingerprint(),
                ArtifactRetention::Full,
            )?;
            let executor = CloudTerrastodonExecutor::new(cache, authentication.clone(), scope);
            let rejected = executor
                .cached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("changed-authentication"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    WorkReuse::ReusableRead,
                    ArtifactRetention::Full,
                    async |child, entry| {
                        let (selected, observation) =
                            publish_number(&entry, 42, child.cancellation()).await?;
                        assert!(selected);
                        authentication
                            .partition_generation
                            .store(1, Ordering::SeqCst);
                        Ok(WorkOutput::new(42_u64, observation))
                    },
                )?
                .await;
            assert!(
                rejected
                    .unwrap_err()
                    .to_string()
                    .contains("authentication identity changed")
            );

            // Restoring the original identity must not expose the rejected
            // outcome through either the original disk entry or a memory hit.
            authentication
                .partition_generation
                .store(0, Ordering::SeqCst);
            assert!(original.load(executor.cancellation()).await?.is_none());
            let starts = AtomicUsize::new(0);
            let recovered = executor
                .cached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("changed-authentication"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    WorkReuse::ReusableRead,
                    ArtifactRetention::Full,
                    async |child, entry| {
                        assert!(entry.load(child.cancellation()).await?.is_none());
                        starts.fetch_add(1, Ordering::SeqCst);
                        Ok(WorkOutput::new(99_u64, ArtifactObservation::now()))
                    },
                )?
                .await?;
            assert_eq!(*recovered, 99);
            assert_eq!(starts.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rejected_nonreusable_artifact_child_prevents_parent_fallback_reuse() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            for (reuse, validity) in [
                (WorkReuse::ExecuteEveryTime, Duration::from_secs(3600)),
                (WorkReuse::ReusableRead, Duration::ZERO),
            ] {
                let cache = WorkCache::temporary()?;
                let root = cache.root().to_path_buf();
                let executor = CloudTerrastodonExecutor::new(
                    cache,
                    Arc::new(TestAuthenticationProvider::offline(
                        "synthetic-test-session",
                    )?),
                    scope.clone(),
                );
                let parent_starts = AtomicUsize::new(0);
                let child_starts = AtomicUsize::new(0);
                let execute = || {
                    executor.cached_work(
                        ExecutorKind::InProcess,
                        key("composition"),
                        CacheOperationOwner::of::<CompositionOperation>(),
                        &AUTHENTICATION,
                        fingerprint(),
                        WorkReuse::ReusableRead,
                        async |parent| {
                            let fallback = parent_starts.fetch_add(1, Ordering::SeqCst) as u64;
                            let rejected = parent.cached_work_with_artifacts(
                                ExecutorKind::Command,
                                CacheKey {
                                    valid_for: validity,
                                    ..key("nonreusable-child")
                                },
                                CacheOperationOwner::of::<NumberOperation>(),
                                &AUTHENTICATION,
                                fingerprint(),
                                reuse,
                                ArtifactRetention::Full,
                                async |_, _| {
                                    child_starts.fetch_add(1, Ordering::SeqCst);
                                    Ok(WorkOutput::new(42_u64, ArtifactObservation::now()))
                                },
                            );
                            assert!(
                                rejected
                                    .err()
                                    .expect("a reusable parent cannot admit a nonreusable child")
                                    .to_string()
                                    .contains("nonreusable")
                            );
                            // Catching the error preserves this caller's fallback,
                            // but cannot make that partial composition reusable.
                            Ok(WorkOutput::new(fallback, ArtifactObservation::now()))
                        },
                    )
                };
                assert_eq!(*execute()?.await?, 0);
                assert_eq!(*execute()?.await?, 1);
                assert_eq!(parent_starts.load(Ordering::SeqCst), 2);
                assert_eq!(child_starts.load(Ordering::SeqCst), 0);
                assert!(std::fs::read_dir(root)?.next().is_none());
            }
            Ok(())
        })
        .await
}
