use crate::AuthContext;
use crate::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AuthSource;
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
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::timeout;

#[derive(Debug, Clone, Copy)]
enum DirectWorkApi {
    Cached,
    CachedArtifacts,
    Uncached,
    UncachedArtifacts,
}

impl DirectWorkApi {
    const ALL: [Self; 4] = [
        Self::Cached,
        Self::CachedArtifacts,
        Self::Uncached,
        Self::UncachedArtifacts,
    ];

    // Exercise every public work boundary with the same lifecycle scenario.
    // The fixture does not replace execution, authentication, or artifact I/O.
    fn execute<'a, F, Fut>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        selection: &'a AuthenticationSelection,
        reuse: WorkReuse,
        execute: F,
    ) -> Result<ExecutionFuture<'a, u64>>
    where
        F: FnOnce(CloudTerrastodonExecutor, Option<ArtifactEntry>) -> Fut + Send + 'a,
        Fut: Future<Output = Result<u64>> + Send + 'a,
    {
        let key = CacheKey {
            scope: CacheScope::try_from_components(["synthetic", "direct-retirement"])?,
            valid_for: Duration::from_secs(60),
        };
        let owner = CacheOperationOwner::of::<Self>();
        let fingerprint = NonEmptyString::new("synthetic-direct-work")?;
        match self {
            Self::Cached => {
                let work = executor.cached_work(
                    ExecutorKind::InProcess,
                    key,
                    owner,
                    selection,
                    fingerprint,
                    reuse,
                    async move |child| {
                        let value = execute(child, None).await?;
                        Ok(WorkOutput::new(value, ArtifactObservation::now()))
                    },
                )?;
                Ok(Box::pin(async move { work.await.map(|value| *value) }))
            }
            Self::CachedArtifacts => {
                let work = executor.cached_work_with_artifacts(
                    ExecutorKind::InProcess,
                    key,
                    owner,
                    selection,
                    fingerprint,
                    reuse,
                    ArtifactRetention::Full,
                    async move |child, entry| {
                        let value = execute(child, Some(entry)).await?;
                        Ok(WorkOutput::new(value, ArtifactObservation::now()))
                    },
                )?;
                Ok(Box::pin(async move { work.await.map(|value| *value) }))
            }
            Self::Uncached => executor.uncached_work(selection, move |child| execute(child, None)),
            Self::UncachedArtifacts => executor.uncached_work_with_artifacts(
                ExecutorKind::InProcess,
                key,
                owner,
                selection,
                fingerprint,
                ArtifactRetention::Full,
                move |child, entry| execute(child, Some(entry)),
            ),
        }
    }
}

#[tokio::test]
async fn unpolled_direct_work_does_not_delay_retirement_or_return_warm_results() -> Result<()> {
    for api in DirectWorkApi::ALL {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let account_id = context.account_id()?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id };
        let provider = Arc::new(CredentialAuthenticationProvider::new(context)?);
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let cache = WorkCache::temporary()?;
                let executor =
                    CloudTerrastodonExecutor::new(cache.clone(), provider.clone(), scope.clone());
                let warm = api
                    .execute(
                        &executor,
                        &selection,
                        WorkReuse::ReusableRead,
                        async |_, _| Ok(7),
                    )?
                    .await?;
                assert_eq!(warm, 7);
                let initialized = AtomicBool::new(false);
                let delayed = api.execute(
                    &executor,
                    &selection,
                    WorkReuse::ReusableRead,
                    async |_, _| {
                        initialized.store(true, Ordering::SeqCst);
                        Ok(9)
                    },
                )?;
                timeout(
                    Duration::from_secs(10),
                    provider.retire(account_id, &cache, &scope),
                )
                .await??;
                assert!(
                    delayed.await.is_err(),
                    "{api:?} returned a result after retirement"
                );
                assert!(!initialized.load(Ordering::SeqCst));
                assert!(!scope.is_cancelled());
                eyre::Ok(())
            })
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn direct_work_retirement_cancels_optional_initializers() -> Result<()> {
    for api in DirectWorkApi::ALL {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let account_id = context.account_id()?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id };
        let provider = Arc::new(CredentialAuthenticationProvider::new(context)?);
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let cache = WorkCache::temporary()?;
                let executor =
                    CloudTerrastodonExecutor::new(cache.clone(), provider.clone(), scope.clone());
                let started = Notify::new();
                let work = api.execute(
                    &executor,
                    &selection,
                    WorkReuse::ReusableRead,
                    async |_, _| {
                        started.notify_one();
                        std::future::pending().await
                    },
                )?;
                let retire = async {
                    started.notified().await;
                    provider.retire(account_id, &cache, &scope).await
                };
                let (work, retired) = timeout(Duration::from_secs(10), async {
                    tokio::join!(work, retire)
                })
                .await?;
                assert!(work.is_err(), "{api:?} did not observe retirement");
                retired?;
                assert!(
                    !scope.is_cancelled(),
                    "retirement cancelled unrelated caller work"
                );
                eyre::Ok(())
            })
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn direct_work_retirement_drains_required_completion_and_artifacts() -> Result<()> {
    for api in DirectWorkApi::ALL {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let account_id = context.account_id()?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id };
        let provider = Arc::new(CredentialAuthenticationProvider::new(context)?);
        CancellationToken::new().with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            let executor = CloudTerrastodonExecutor::new(cache.clone(), provider.clone(), scope.clone());
            let started = Notify::new();
            let finish = Notify::new();
            let completed = AtomicBool::new(false);
            let retained_artifact = Mutex::new(None);
            let work = executor.deferred("synthetic mutation with required completion", async |protected| {
                api.execute(&protected, &selection, WorkReuse::ExecuteEveryTime, async |child, entry| {
                    started.notify_one();
                    finish.notified().await;
                    child.admit(&selection)?;
                    if let Some(entry) = entry {
                        let stored = entry.begin_attempt(BTreeMap::new(), child.cancellation()).await?
                            .expect("full retention stages an attempt")
                            .finish(
                                ArtifactSet { payload: b"synthetic completed mutation".to_vec(), diagnostics: BTreeMap::new() },
                                ArtifactObservation::now(),
                                child.cancellation(),
                            ).await?;
                        *retained_artifact.lock().unwrap() = Some(stored.directory().to_path_buf());
                        assert!(!entry.publish(&stored, child.cancellation()).await?, "a mutation cannot publish reusable data");
                    }
                    child.cache()?.invalidate(
                        &CacheScope::try_from_components(["synthetic", "affected-read"] )?,
                        child.cancellation(),
                    ).await?;
                    completed.store(true, Ordering::SeqCst);
                    Ok(42)
                })?.await
            });
            let retire = async {
                started.notified().await;
                let retirement = provider.retire(account_id, &cache, &scope);
                tokio::pin!(retirement);
                tokio::select! {
                    result = &mut retirement => panic!("{api:?} retired before required completion: {result:?}"),
                    _ = tokio::task::yield_now() => {}
                }
                assert!(provider.admit(&selection).is_err());
                assert!(!completed.load(Ordering::SeqCst));
                finish.notify_one();
                retirement.await
            };
            let (work, retired) = timeout(Duration::from_secs(10), async {
                tokio::join!(work, retire)
            }).await?;
            assert_eq!(work?, 42);
            retired?;
            assert!(completed.load(Ordering::SeqCst));
            if matches!(api, DirectWorkApi::CachedArtifacts | DirectWorkApi::UncachedArtifacts) {
                assert!(retained_artifact.lock().unwrap().as_ref().unwrap().exists());
            }
            assert!(!scope.is_cancelled());
            eyre::Ok(())
        }).await?;
    }
    Ok(())
}
