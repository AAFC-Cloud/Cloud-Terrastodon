use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerGetRequest;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerName;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// Show one blob container's management-plane properties.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageBlobContainerShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Full ARM resource ID of the storage account.
    #[facet(figue::named, proxy = String)]
    pub storage_account: AzureStorageAccountId,
    /// Exact container name, including special names such as $web or $root.
    #[facet(figue::named, proxy = String)]
    pub container: AzureStorageAccountBlobContainerName,
    /// Refresh the selected container's cached properties.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureStorageBlobContainerShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let container = AzureStorageAccountBlobContainerGetRequest {
            authentication: self.authentication.resolve(executor).await?,
            container_id: Cow::Owned(AzureStorageAccountBlobContainerId {
                storage_account_id: self.storage_account,
                container_name: self.container,
            }),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(container))
    }
}
