use crate::AzureDevOpsCliConfigRequest;
use crate::get_azure_devops_cli_config;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::OptionExt;

/// Inspect the project selected in local Azure CLI configuration.
/// Service requests still require their own explicit project.
#[derive(Debug, Clone, Copy, Arbitrary, facet::Facet)]
pub struct DefaultAzureDevOpsProjectNameRequest;

/// Read the project from fresh local configuration without managed reuse.
pub async fn get_default_project_name(
    cancellation: &CancellationScope,
) -> eyre::Result<AzureDevOpsProjectName> {
    DefaultAzureDevOpsProjectNameRequest::parse_config(
        &get_azure_devops_cli_config(cancellation).await?,
    )
}
impl DefaultAzureDevOpsProjectNameRequest {
    fn parse_config(config: &str) -> eyre::Result<AzureDevOpsProjectName> {
        let project = config.lines()
            .filter_map(|line| line.split_once('='))
            .find_map(|(key, value)| (key.trim() == "project").then_some(value.trim()))
            .ok_or_eyre("Expected a project configured using `az devops configure --defaults project=MyProject`")?;
        AzureDevOpsProjectName::try_new(project.to_owned())
    }
}

impl CacheInvalidationKeys for DefaultAzureDevOpsProjectNameRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![AzureDevOpsCliConfigRequest.cache_key()?])
    }
}

impl ExecutableRequest for DefaultAzureDevOpsProjectNameRequest {
    type Output = AzureDevOpsProjectName;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let config = executor.execute(AzureDevOpsCliConfigRequest).await?;
            Self::parse_config(&config)
        })
    }
}
cloud_terrastodon_registry::register_thing!(DefaultAzureDevOpsProjectNameRequest);
cloud_terrastodon_registry::register_arbitrary!(DefaultAzureDevOpsProjectNameRequest);
cloud_terrastodon_registry_execution::register_executable_request!(DefaultAzureDevOpsProjectNameRequest => AzureDevOpsProjectName, effects = [Read]);
