#[cfg(feature = "auth")]
use cloud_terrastodon_auth::AuthContext;
#[cfg(feature = "auth")]
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
#[cfg(not(feature = "auth"))]
use cloud_terrastodon_execution::DisabledAuthenticationProvider;
use std::sync::Arc;
use teamy_cancellation::CancellationScope;

/// Facilities owned by this invocation, not process-global application state.
pub struct AppContext {
    /// The handler's execution scope, including the installed Ctrl-C handler.
    pub cancellation: CancellationScope,
    /// Resolved authentication preferences; does not itself request a token.
    #[cfg(feature = "auth")]
    pub auth: AuthContext,
    // Every executor bound by this context shares one cache owner. Keep the
    // owner private so repeated bindings cannot silently replace that lifetime.
    cache: WorkCache,
}

impl AppContext {
    pub(crate) fn new(
        cache: WorkCache,
        cancellation: CancellationScope,
        #[cfg(feature = "auth")] auth: AuthContext,
    ) -> Self {
        Self {
            cancellation,
            #[cfg(feature = "auth")]
            auth,
            cache,
        }
    }

    /// Bind a cheap executor handle to the context's current execution resources.
    ///
    /// The cache owner and resolved credential session are shared across calls.
    /// Binding performs no credential discovery, token acquisition, or cache I/O.
    /// The legacy public authentication and cancellation fields remain the source
    /// of truth until callers migrate; no second stored executor can disagree
    /// with them. An already-returned executor retains its original binding.
    pub fn executor(&self) -> CloudTerrastodonExecutor {
        #[cfg(feature = "auth")]
        let authentication = Arc::new(CredentialAuthenticationProvider::new(self.auth.clone()));
        #[cfg(not(feature = "auth"))]
        let authentication = Arc::new(DisabledAuthenticationProvider);
        CloudTerrastodonExecutor::new(
            self.cache.clone(),
            authentication,
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
    async fn executor_bindings_share_cache_but_follow_explicit_scope_replacement()
    -> eyre::Result<()> {
        let original_signal = CancellationToken::new();
        let replacement_signal = CancellationToken::new();
        original_signal
            .with_async_scope(|original_scope| async {
                replacement_signal
                    .with_async_scope(|replacement_scope| async {
                        // Construction only validates the path. This test never
                        // reads, creates, or executes work against this cache.
                        let mut context = AppContext::new(
                            WorkCache::new(
                                std::env::temp_dir()
                                    .join("cloud-terrastodon-app-context-test-cache"),
                            )?,
                            original_scope,
                            #[cfg(feature = "auth")]
                            AuthContext::None,
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

    #[cfg(feature = "auth")]
    #[tokio::test]
    async fn executor_bindings_preserve_session_identity_and_follow_auth_replacement()
    -> eyre::Result<()> {
        use cloud_terrastodon_auth_types::AuthSource;
        use cloud_terrastodon_auth_types::AuthenticationSelection;

        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let mut context = AppContext::new(
                    WorkCache::new(
                        std::env::temp_dir().join("cloud-terrastodon-app-context-test-cache"),
                    )?,
                    cancellation,
                    AuthContext::explicit(AuthSource::PersonalAccessToken),
                );
                let original = context.executor();
                let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken;
                let partition = original.admit(&selection)?;
                assert_eq!(context.executor().admit(&selection)?, partition);

                context.auth = AuthContext::None;
                assert!(context.executor().admit(&selection).is_err());
                assert_eq!(original.admit(&selection)?, partition);
                Ok(())
            })
            .await
    }
}
