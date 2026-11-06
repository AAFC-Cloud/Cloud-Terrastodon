use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::sync::Arc;

/// Tests own fresh artifacts and explicitly select their authentication policy.
/// Construction does not read configuration or acquire a credential.
pub(crate) fn test_executor(
    authentication: &AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> eyre::Result<CloudTerrastodonExecutor> {
    let authentication = match authentication {
        AzureDevOpsAuthContext::None => AuthContext::explicit(AuthSource::None),
        AzureDevOpsAuthContext::Bearer(context) => context.auth_context.clone(),
        AzureDevOpsAuthContext::PersonalAccessToken(context) => context.clone(),
    };
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(CredentialAuthenticationProvider::new(authentication)),
        cancellation.clone(),
    ))
}
