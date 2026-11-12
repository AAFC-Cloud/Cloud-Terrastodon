use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::sync::Arc;
use teamy_cancellation::CancellationScope;

/// Facilities owned by this invocation, not process-global application state.
pub struct AppContext {
    /// The handler's execution scope, including the installed Ctrl-C handler.
    pub cancellation: CancellationScope,
    // Keep the application-owned provider shared; requests select accounts, while
    // every executor observes the same credential sessions and retirement state.
    authentication: Arc<dyn AuthenticationProvider>,
    // Every executor created by this context shares one cache owner. Keep the
    // owner private so creating another handle cannot replace that lifetime.
    cache: WorkCache,
}

impl AppContext {
    pub(crate) fn new(
        cache: WorkCache,
        cancellation: CancellationScope,
        authentication: Arc<dyn AuthenticationProvider>,
    ) -> Self {
        Self {
            cancellation,
            authentication,
            cache,
        }
    }

    /// Create a cheap executor handle sharing this invocation's execution resources.
    ///
    /// The cache owner and authentication provider are shared across calls.
    /// Construction performs no credential discovery, token acquisition, or cache
    /// I/O. An already-returned executor retains its original cancellation scope.
    pub fn executor(&self) -> CloudTerrastodonExecutor {
        CloudTerrastodonExecutor::new(
            self.cache.clone(),
            Arc::clone(&self.authentication),
            self.cancellation.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::CacheKey;
    use cloud_terrastodon_cache::CacheOperationOwner;
    use cloud_terrastodon_cache::CacheScope;
    use cloud_terrastodon_cache::ExecutorKind;
    use std::time::Duration;
    use teamy_cancellation::CancellationToken;

    #[tokio::test]
    async fn executors_reuse_the_injected_authentication_provider() -> eyre::Result<()> {
        use cloud_terrastodon_auth_types::AuthenticationSelection;
        use cloud_terrastodon_execution::TestAuthenticationProvider;
        use std::sync::atomic::Ordering;

        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let authentication =
                    Arc::new(TestAuthenticationProvider::offline("application-provider")?);
                let context = AppContext::new(
                    WorkCache::temporary()?,
                    cancellation,
                    authentication.clone(),
                );
                let selection = AuthenticationSelection::Anonymous;
                let partition = context.executor().admit(&selection)?;
                assert_eq!(context.executor().admit(&selection)?, partition);
                assert_eq!(authentication.admissions.load(Ordering::SeqCst), 2);
                assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn executor_bindings_share_cache_but_follow_explicit_scope_replacement()
    -> eyre::Result<()> {
        let original_signal = CancellationToken::new();
        let replacement_signal = CancellationToken::new();
        original_signal
            .with_async_scope(|original_scope| async {
                replacement_signal
                    .with_async_scope(|replacement_scope| async {
                        let mut context = AppContext::new(
                            WorkCache::temporary()?,
                            original_scope,
                            Arc::new(cloud_terrastodon_execution::AnonymousAuthenticationProvider),
                        );
                        let original = context.executor();
                        context.cancellation = replacement_scope;
                        let rebound = context.executor();

                        let key = CacheKey {
                            scope: CacheScope::try_from_components(["synthetic", "local-value"])?,
                            valid_for: Duration::from_secs(60),
                        };
                        original.cache()?.claim::<u32>(
                            ExecutorKind::InProcess,
                            &key,
                            CacheOperationOwner::of::<u32>(),
                        )?;
                        // A second binding must see the first binding's owner
                        // contract, rather than creating an independent cache.
                        assert!(
                            rebound
                                .cache()?
                                .claim::<String>(
                                    ExecutorKind::InProcess,
                                    &key,
                                    CacheOperationOwner::of::<u32>(),
                                )
                                .is_err()
                        );

                        original_signal.request_cancel("original handler cancelled");
                        assert!(original.cancellation().is_cancelled());
                        assert!(!rebound.cancellation().is_cancelled());

                        replacement_signal.request_cancel("replacement scope cancelled");
                        assert!(rebound.cancellation().is_cancelled());
                        Ok(())
                    })
                    .await
            })
            .await
    }
}
