use arbitrary::Arbitrary;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::RetryBehaviour;
use cloud_terrastodon_command::bstr::ByteSlice;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use std::time::Duration;

/// Read local Azure CLI extension configuration; this performs no service request.
/// <https://learn.microsoft.com/cli/azure/devops#az-devops-configure>
#[derive(Debug, Clone, Copy, Arbitrary, facet::Facet)]
pub struct AzureDevOpsCliConfigRequest;

/// Read fresh local configuration without cache or artifact management.
/// Execute [`AzureDevOpsCliConfigRequest`] with an executor for managed reuse.
pub async fn get_azure_devops_cli_config(cancellation: &CancellationScope) -> eyre::Result<String> {
    let output = AzureDevOpsCliConfigRequest::command()
        .run_raw(cancellation)
        .await?;
    AzureDevOpsCliConfigRequest::decode_config(&output)
}

impl AzureDevOpsCliConfigRequest {
    fn command() -> CommandBuilder {
        let mut command = CommandBuilder::new(CommandKind::AzureCLI);
        command
            .cancellation_safe()
            .args(["devops", "configure", "--list"])
            .use_retry_behaviour(RetryBehaviour::Fail);
        command
    }

    fn decode_config(output: &CommandOutput) -> eyre::Result<String> {
        Ok(output.stdout.to_str()?.to_owned())
    }
}

impl HasCacheKey for AzureDevOpsCliConfigRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components(["az", "devops", "config", "list"])?,
            valid_for: Duration::from_secs(30),
        })
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsCliConfigRequest {}

impl ExecutableRequest for AzureDevOpsCliConfigRequest {
    type Error = eyre::Report;
    type Output = String;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let output = Self::command()
                .cache(self.cache_key()?)
                .run_raw_cached(&executor, CacheOperationOwner::of::<Self>())
                .await?;
            Self::decode_config(&output)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsCliConfigRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsCliConfigRequest);
cloud_terrastodon_registry::register_arbitrary!(String);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsCliConfigRequest => String, effects = [Read]);
