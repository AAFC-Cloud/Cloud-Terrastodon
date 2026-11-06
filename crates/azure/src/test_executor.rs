use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::sync::Arc;

/// Build an explicitly selected test runtime without consulting credentials or
/// configuration. Each test owns its temporary artifacts and supplied scope.
pub(crate) fn test_executor(
    authentication: &AuthContext,
    cancellation: &CancellationScope,
) -> eyre::Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(CredentialAuthenticationProvider::new(
            authentication.clone(),
        )),
        cancellation.clone(),
    ))
}
