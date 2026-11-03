use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::sync::Arc;

/// Bind local test execution to an owned temporary cache.
pub(crate) fn synthetic_executor(cancellation: &CancellationScope) -> CloudTerrastodonExecutor {
    CloudTerrastodonExecutor::new(
        WorkCache::temporary().unwrap(),
        Arc::new(AnonymousAuthenticationProvider),
        cancellation.clone(),
    )
}
