use arbitrary::Arbitrary;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::RetryBehaviour;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::bstr::ByteSlice;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct AzureDevOpsCliConfigRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}

pub fn get_azure_devops_cli_config(
    cancellation: &CancellationScope,
) -> AzureDevOpsCliConfigRequest {
    AzureDevOpsCliConfigRequest {
        cancellation: cancellation.clone(),
    }
}

#[async_trait]
impl CacheableCommand for AzureDevOpsCliConfigRequest {
    type Output = String;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "devops", "config", "list"]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.cancellation_safe();
        cmd.args(["devops", "configure", "--list"]);
        // Configuration is local metadata; it must never turn an unconfigured
        // Azure CLI into an implicit interactive login.
        cmd.use_retry_behaviour(RetryBehaviour::Fail);
        cmd.cache(self.cache_key());
        let rtn = cmd.run_raw(&self.cancellation).await?;
        let stdout = rtn.stdout.to_str()?.to_string();
        Ok(stdout)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsCliConfigRequest);
cloud_terrastodon_registry::register_thing!(AzureDevOpsCliConfigRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsCliConfigRequest);
cloud_terrastodon_registry::register_arbitrary!(String);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsCliConfigRequest => String,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::get_azure_devops_cli_config;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let config = get_azure_devops_cli_config(&cancellation).await?;
                assert!(
                    !config.trim().is_empty(),
                    "Expected Azure DevOps CLI config output"
                );
                Ok(())
            })
            .await
    }
}
