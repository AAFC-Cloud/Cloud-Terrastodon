use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use std::sync::Arc;

/// Offline executor for local request validation, cache keys, and fake writers.
pub(crate) fn test_executor(
    cancellation: &CancellationScope,
) -> eyre::Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(TestAuthenticationProvider::offline(
            "devops-request-policy",
        )?),
        cancellation.clone(),
    ))
}

pub(crate) fn authentication_selection() -> AzureDevOpsAuthSelection {
    AzureDevOpsAuthSelection::PersonalAccessToken {
        account_id: "00000000-0000-0000-0000-000000000001"
            .parse()
            .expect("a synthetic account ID"),
    }
}
