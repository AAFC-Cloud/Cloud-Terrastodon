use crate::AzureDevOpsCliConfigRequest;
use crate::get_azure_devops_cli_config;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::OptionExt;
use facet::Facet;
use tracing::info;

/// Inspect the organization selected in local Azure CLI configuration.
/// Service requests still require their own explicit organization.
#[derive(Debug, Clone, Copy, Arbitrary, Facet)]
pub struct DefaultAzureDevOpsOrganizationUrlRequest;

/// Read the organization from fresh local configuration without managed reuse.
pub async fn get_default_organization_url(
    cancellation: &CancellationScope,
) -> eyre::Result<AzureDevOpsOrganizationUrl> {
    DefaultAzureDevOpsOrganizationUrlRequest::parse_config(
        &get_azure_devops_cli_config(cancellation).await?,
    )
}

impl DefaultAzureDevOpsOrganizationUrlRequest {
    fn parse_config(config: &str) -> eyre::Result<AzureDevOpsOrganizationUrl> {
        let org = config.lines()
            .filter_map(|line| line.split_once('='))
            .find_map(|(key, value)| (key.trim() == "organization").then_some(value.trim()))
            .ok_or_eyre("Expected an organization configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`")?;
        org.parse()
    }
}

impl CacheInvalidationKeys for DefaultAzureDevOpsOrganizationUrlRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![AzureDevOpsCliConfigRequest.cache_key()?])
    }
}

impl ExecutableRequest for DefaultAzureDevOpsOrganizationUrlRequest {
    type Error = eyre::Report;
    type Output = AzureDevOpsOrganizationUrl;
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

/// Update local CLI configuration and invalidate its cached inspection.
/// <https://learn.microsoft.com/cli/azure/devops#az-devops-configure>
pub async fn set_default_organization_url(
    org: AzureDevOpsOrganizationUrl,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    info!("Setting default organization to {}", org);
    executor
        .deferred(
            "setting default organization and invalidating cached configuration",
            |executor| async move {
                let result = CommandBuilder::new(CommandKind::AzureCLI)
                    .args([
                        "devops",
                        "configure",
                        "--defaults",
                        &format!("organization={org}"),
                    ])
                    .run_raw_with_artifacts(&executor)
                    .await;
                // Artifact completion may fail after the CLI has changed its defaults.
                let invalidation = AzureDevOpsCliConfigRequest.invalidate(&executor).await;
                match (result, invalidation) {
                    (Ok(_), Ok(())) => Ok(()),
                    (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                        "cached CLI configuration invalidation also failed: {invalidation:#}"
                    ))),
                }
            },
        )
        .await
}

cloud_terrastodon_registry::register_thing!(DefaultAzureDevOpsOrganizationUrlRequest);
cloud_terrastodon_registry::register_arbitrary!(DefaultAzureDevOpsOrganizationUrlRequest);
cloud_terrastodon_registry_execution::register_executable_request!(DefaultAzureDevOpsOrganizationUrlRequest => AzureDevOpsOrganizationUrl, effects = [Read]);
