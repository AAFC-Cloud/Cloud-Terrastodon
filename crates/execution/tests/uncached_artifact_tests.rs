#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::ArtifactEntry;
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
use eyre::Result;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::Barrier;
use tokio::time::timeout;

const AUTHENTICATION: AuthenticationSelection =
    AuthenticationSelection::AzureDevOpsPersonalAccessToken;
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct NumberOperation;
struct OtherOperation;
struct CompositionOperation;

#[derive(Default)]
struct SyntheticAuthentication {
    generation: AtomicUsize,
}

impl AuthenticationProvider for SyntheticAuthentication {
    fn admit(&self, _selection: &AuthenticationSelection) -> Result<AuthPartition> {
        AuthPartition::new(format!(
            "synthetic-session-{}",
            self.generation.load(Ordering::SeqCst)
        ))
    }

    fn authorization<'a>(
        &'a self,
        _selection: &'a AuthenticationSelection,
        _resource: AzureCredentialResource,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async {
            eyre::bail!("synthetic artifact work must not acquire credentials");
        })
    }
}

#[derive(Default)]
struct CredentialWorkProbe {
    starts: AtomicUsize,
}

impl AuthenticationProvider for CredentialWorkProbe {
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
            let ordinary = executor.uncached_work(selection, |_| async {
                self.starts.fetch_add(1, Ordering::SeqCst);
                Ok(Cell::new(7_u64))
            });
            assert!(
                ordinary
                    .err()
                    .expect("credential infrastructure cannot admit ordinary work")
                    .to_string()
                    .contains("credential infrastructure")
            );
            let artifacts = executor.uncached_work_with_artifacts(
                ExecutorKind::Command,
                key("credential-infrastructure"),
                CacheOperationOwner::of::<NumberOperation>(),
                selection,
                fingerprint(),
                ArtifactRetention::Full,
                |_, _| async {
                    self.starts.fetch_add(1, Ordering::SeqCst);
                    Ok(Cell::new(7_u64))
                },
            );
            assert!(
                artifacts
                    .err()
                    .expect("credential infrastructure cannot bind execution artifacts")
                    .to_string()
                    .contains("credential infrastructure")
            );
            Ok(Authorization::Anonymous)
        })
    }
}

fn key(name: &str) -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "uncached-artifacts", name]).unwrap(),
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
        .expect("full retention must stage execution artifacts")
        .finish(
            ArtifactSet {
                payload: value.to_le_bytes().to_vec(),
                diagnostics: BTreeMap::new(),
            },
            observation,
            cancellation,
        )
        .await?;
    assert!(stored.directory().exists());
    Ok((entry.publish(&stored, cancellation).await?, observation))
}

#[tokio::test]
async fn non_sync_outputs_execute_independently_without_reusing_or_selecting_disk_results()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let authentication = Arc::new(SyntheticAuthentication::default());
            let prior = cache.artifacts::<Cell<u64>>(
                ExecutorKind::Command,
                key("independent-attempts"),
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
                let starts = &starts;
                let simultaneous = &simultaneous;
                executor.uncached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("independent-attempts"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    ArtifactRetention::Full,
                    move |child, entry| async move {
                        assert!(entry.load(child.cancellation()).await?.is_none());
                        let number = starts.fetch_add(1, Ordering::SeqCst) as u64 + 10;
                        // Per-key joining or serialization would deadlock here.
                        simultaneous.wait().await;
                        assert!(
                            !publish_number(&entry, number, child.cancellation())
                                .await?
                                .0
                        );
                        Ok(Cell::new(number))
                    },
                )
            };
            let first = execute()?;
            let second = execute()?;
            let (first, second) =
                timeout(DEADLOCK_GUARD, async { tokio::try_join!(first, second) }).await??;
            assert_ne!(first.get(), second.get());
            first.set(99);
            assert_ne!(first.get(), second.get());
            assert_eq!(starts.load(Ordering::SeqCst), 2);
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
async fn artifact_contract_conflicts_fail_at_binding_before_execution_or_disk_access() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let executor = CloudTerrastodonExecutor::new(
                cache,
                Arc::new(SyntheticAuthentication::default()),
                scope,
            );
            let starts = AtomicUsize::new(0);
            let first = executor.uncached_work_with_artifacts(
                ExecutorKind::Command,
                key("contract"),
                CacheOperationOwner::of::<NumberOperation>(),
                &AUTHENTICATION,
                fingerprint(),
                ArtifactRetention::Full,
                |_, _| async {
                    starts.fetch_add(1, Ordering::SeqCst);
                    Ok(Cell::new(7_u64))
                },
            )?;
            let owner_conflict = executor.uncached_work_with_artifacts(
                ExecutorKind::Command,
                key("contract"),
                CacheOperationOwner::of::<OtherOperation>(),
                &AUTHENTICATION,
                fingerprint(),
                ArtifactRetention::Full,
                |_, _| async {
                    starts.fetch_add(1, Ordering::SeqCst);
                    Ok(Cell::new(8_u64))
                },
            );
            assert!(
                owner_conflict
                    .err()
                    .expect("a logical operation cannot change owner")
                    .to_string()
                    .contains("owner conflict")
            );
            let type_conflict = executor.uncached_work_with_artifacts(
                ExecutorKind::Command,
                key("contract"),
                CacheOperationOwner::of::<NumberOperation>(),
                &AUTHENTICATION,
                fingerprint(),
                ArtifactRetention::Full,
                |_, _| async {
                    starts.fetch_add(1, Ordering::SeqCst);
                    Ok(9_u64)
                },
            );
            assert!(
                type_conflict
                    .err()
                    .expect("owned outputs still have one canonical type")
                    .to_string()
                    .contains("output type conflict")
            );
            drop(first);
            assert_eq!(starts.load(Ordering::SeqCst), 0);
            assert!(std::fs::read_dir(root)?.next().is_none());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn caught_uncached_child_rejection_quarantines_an_already_selected_parent() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let authentication = Arc::new(SyntheticAuthentication::default());
            let parent_artifacts = cache.artifacts::<u64>(
                ExecutorKind::InProcess,
                key("composition"),
                CacheOperationOwner::of::<CompositionOperation>(),
                authentication.admit(&AUTHENTICATION)?,
                fingerprint(),
                ArtifactRetention::Full,
            )?;
            let executor = CloudTerrastodonExecutor::new(cache, authentication, scope);
            let parent_starts = AtomicUsize::new(0);
            let child_starts = AtomicUsize::new(0);
            let execute = || {
                let parent_starts = &parent_starts;
                let child_starts = &child_starts;
                executor.cached_work_with_artifacts(
                    ExecutorKind::InProcess,
                    key("composition"),
                    CacheOperationOwner::of::<CompositionOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    WorkReuse::ReusableRead,
                    ArtifactRetention::Full,
                    move |parent, entry| async move {
                        let fallback = parent_starts.fetch_add(1, Ordering::SeqCst) as u64;
                        assert!(entry.load(parent.cancellation()).await?.is_none());
                        let (selected, observation) =
                            publish_number(&entry, fallback, parent.cancellation()).await?;
                        assert!(selected);
                        let rejected = parent.uncached_work_with_artifacts(
                            ExecutorKind::Command,
                            key("uncached-child"),
                            CacheOperationOwner::of::<NumberOperation>(),
                            &AUTHENTICATION,
                            fingerprint(),
                            ArtifactRetention::Full,
                            |_, _| async move {
                                child_starts.fetch_add(1, Ordering::SeqCst);
                                Ok(Cell::new(42_u64))
                            },
                        );
                        assert!(
                            rejected
                                .err()
                                .expect("reusable parents must reject uncached child work")
                                .to_string()
                                .contains("nonreusable")
                        );
                        Ok(WorkOutput::new(fallback, observation))
                    },
                )
            };
            assert_eq!(*execute()?.await?, 0);
            assert!(
                parent_artifacts
                    .load(executor.cancellation())
                    .await?
                    .is_none()
            );
            assert_eq!(*execute()?.await?, 1);
            assert!(
                parent_artifacts
                    .load(executor.cancellation())
                    .await?
                    .is_none()
            );
            assert_eq!(parent_starts.load(Ordering::SeqCst), 2);
            assert_eq!(child_starts.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn authentication_changes_reject_bound_work_and_completed_owned_outputs() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let authentication = Arc::new(SyntheticAuthentication::default());
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let starts = AtomicUsize::new(0);
            let pending = executor.uncached_work_with_artifacts(
                ExecutorKind::Command,
                key("authentication"),
                CacheOperationOwner::of::<NumberOperation>(),
                &AUTHENTICATION,
                fingerprint(),
                ArtifactRetention::Full,
                |_, _| async {
                    starts.fetch_add(1, Ordering::SeqCst);
                    Ok(Cell::new(7_u64))
                },
            )?;
            authentication.generation.store(1, Ordering::SeqCst);
            assert!(
                pending
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("authentication identity changed")
            );
            assert_eq!(starts.load(Ordering::SeqCst), 0);
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_none()
            );

            let completed = executor
                .uncached_work_with_artifacts(
                    ExecutorKind::Command,
                    key("authentication"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    fingerprint(),
                    ArtifactRetention::Full,
                    |child, entry| {
                        let authentication = &authentication;
                        let starts = &starts;
                        async move {
                            starts.fetch_add(1, Ordering::SeqCst);
                            assert!(!publish_number(&entry, 8, child.cancellation()).await?.0);
                            authentication.generation.store(2, Ordering::SeqCst);
                            Ok(Cell::new(8_u64))
                        }
                    },
                )?
                .await;
            assert!(
                completed
                    .unwrap_err()
                    .to_string()
                    .contains("authentication identity changed")
            );
            assert_eq!(starts.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn credential_infrastructure_cannot_enter_either_owned_work_boundary() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let root = cache.root().to_path_buf();
            let provider = Arc::new(CredentialWorkProbe::default());
            let executor = CloudTerrastodonExecutor::new(cache, provider.clone(), scope);
            executor
                .authorization(&AUTHENTICATION, AzureCredentialResource::AzureDevOps)
                .await?;
            assert_eq!(provider.starts.load(Ordering::SeqCst), 0);
            assert!(std::fs::read_dir(root)?.next().is_none());
            Ok(())
        })
        .await
}
