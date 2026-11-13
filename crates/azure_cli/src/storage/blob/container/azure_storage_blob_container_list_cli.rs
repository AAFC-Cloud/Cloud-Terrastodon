use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerListRequest;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List live blob containers and their management-plane properties.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageBlobContainerListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Full ARM resource ID of the storage account.
    #[facet(figue::named, proxy = String)]
    pub storage_account: AzureStorageAccountId,
    /// Refresh every cached page of this account's container inventory.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureStorageBlobContainerListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let containers = AzureStorageAccountBlobContainerListRequest {
            authentication: self.authentication.resolve(executor).await?,
            storage_account_id: Cow::Borrowed(&self.storage_account),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(containers))
    }
}
