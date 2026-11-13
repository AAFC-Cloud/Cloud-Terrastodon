use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureStorageAccountListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List storage accounts and their typed properties through Azure Resource Graph.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageAccountListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Refresh the selected tenant's cached storage-account inventory.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureStorageAccountListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let accounts = AzureStorageAccountListRequest {
            authentication: self.authentication.resolve(executor).await?,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(accounts))
    }
}
