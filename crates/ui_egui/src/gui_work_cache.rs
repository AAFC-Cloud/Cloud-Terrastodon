use cloud_terrastodon_cache::WorkCacheActivity;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::sync::Arc;

/// Selects ownership for GUI jobs that access the work cache.
#[derive(Clone)]
pub(crate) enum GuiWorkCache {
    /// An embedded application supplies an executor with its own cache owner.
    CallerOwned,
    /// Overlapping jobs share an owner while the GUI retains only the activity.
    ActivityScoped(Arc<WorkCacheActivity>),
}

impl GuiWorkCache {
    /// Bind an executor for the cache-bearing part of a tracked job.
    pub fn job_executor(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        match self {
            Self::CallerOwned => Ok(executor.clone()),
            Self::ActivityScoped(activity) => executor.with_work_cache(activity.work_cache()?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AuthenticationSelection;
    use cloud_terrastodon_cache::ArtifactObservation;
    use cloud_terrastodon_cache::CacheKey;
    use cloud_terrastodon_cache::CacheOperationOwner;
    use cloud_terrastodon_cache::CacheScope;
    use cloud_terrastodon_cache::ExecutorKind;
    use cloud_terrastodon_cache::NonEmptyString;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_cache::WorkOutput;
    use cloud_terrastodon_cache::WorkReuse;
    use cloud_terrastodon_command::app_work::AppWorkTracker;
    use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
    use std::time::Duration;
    use teamy_cancellation::CancellationToken;
    use tokio::runtime::Handle;
    use tokio::sync::oneshot;
    use tokio::time::timeout;

    const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

    struct SyntheticInventory;

    async fn load_inventory(
        executor: &CloudTerrastodonExecutor,
        value: usize,
    ) -> Result<Arc<usize>> {
        executor
            .cached_work(
                ExecutorKind::InProcess,
                CacheKey {
                    scope: CacheScope::try_from_components(["synthetic", "gui-inventory"])?,
                    valid_for: Duration::from_secs(60),
                },
                CacheOperationOwner::of::<SyntheticInventory>(),
                &AuthenticationSelection::Anonymous,
                NonEmptyString::new("inventory")?,
                WorkReuse::ReusableRead,
                move |_| async move { eyre::Ok(WorkOutput::new(value, ArtifactObservation::now())) },
            )?
            .await
    }

    #[tokio::test]
    async fn idle_gui_releases_cache_while_displaying_the_last_inventory() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let root = WorkCache::temporary()?;
                let executor = CloudTerrastodonExecutor::new(
                    root.clone(),
                    Arc::new(AnonymousAuthenticationProvider),
                    scope.clone(),
                );
                let work_cache =
                    GuiWorkCache::ActivityScoped(Arc::new(WorkCacheActivity::new(root.root())?));
                let tracker = AppWorkTracker::new();
                let (send, snapshot) = oneshot::channel();
                let base = executor.clone();
                let job_cache = work_cache.clone();
                tracker.spawn(
                    &scope,
                    &Handle::current(),
                    "load synthetic GUI inventory".to_owned(),
                    async move |scope| {
                        let inventory = {
                            let job =
                                job_cache.job_executor(&base.with_cancellation_scope(scope))?;
                            load_inventory(&job, 1).await?
                        };
                        send.send(inventory)
                            .map_err(|_| eyre::eyre!("GUI test stopped polling inventory"))
                    },
                )?;
                let snapshot = timeout(DEADLOCK_GUARD, snapshot).await??;

                // The window, its tracker, and its rendered value all stay alive.
                let command = WorkCache::new(root.root())?;
                timeout(DEADLOCK_GUARD, command.invalidate_all(&scope)).await??;
                drop(command);

                let job = work_cache.job_executor(&executor)?;
                let refreshed = load_inventory(&job, 2).await?;
                assert_eq!(*snapshot, 1);
                assert_eq!(*refreshed, 2);
                tracker.finish(&scope).await
            })
            .await
    }

    #[tokio::test]
    async fn synthetic_gui_sign_in_leaves_the_cache_available_after_retirement() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let root = WorkCache::temporary()?;
                let executor = CloudTerrastodonExecutor::new(
                    root.clone(),
                    Arc::new(AnonymousAuthenticationProvider),
                    scope.clone(),
                );
                let work_cache =
                    GuiWorkCache::ActivityScoped(Arc::new(WorkCacheActivity::new(root.root())?));
                let tracker = AppWorkTracker::new();
                let (prompt, waiting) = oneshot::channel();
                let (resume, sign_in) = oneshot::channel();
                let (finished, outcome) = oneshot::channel();
                tracker.spawn(
                    &scope,
                    &Handle::current(),
                    "simulate GUI retirement and sign-in".to_owned(),
                    async move |scope| {
                        let executor = executor.with_cancellation_scope(scope);
                        {
                            let job = work_cache.job_executor(&executor)?;
                            job.cache()?.invalidate_all(job.cancellation()).await?;
                        }
                        prompt.send(()).map_err(|_| eyre::eyre!("prompt closed"))?;
                        // Stand in for a user completing the prompt, with no credential I/O.
                        sign_in.await?;
                        finished.send(()).map_err(|_| eyre::eyre!("sign-in closed"))
                    },
                )?;
                timeout(DEADLOCK_GUARD, waiting).await??;

                let command = WorkCache::new(root.root())?;
                timeout(DEADLOCK_GUARD, command.acquire(&scope)).await??;
                drop(command);
                resume.send(()).map_err(|_| eyre::eyre!("sign-in closed"))?;
                timeout(DEADLOCK_GUARD, outcome).await??;
                tracker.finish(&scope).await
            })
            .await
    }

    #[tokio::test]
    async fn embedded_gui_uses_its_callers_cache_owner() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(AnonymousAuthenticationProvider),
                    scope,
                );
                let original = load_inventory(&executor, 1).await?;
                let embedded = GuiWorkCache::CallerOwned.job_executor(&executor)?;
                let reused = timeout(DEADLOCK_GUARD, load_inventory(&embedded, 2)).await??;
                assert!(Arc::ptr_eq(&original, &reused));
                eyre::Ok(())
            })
            .await
    }
}
