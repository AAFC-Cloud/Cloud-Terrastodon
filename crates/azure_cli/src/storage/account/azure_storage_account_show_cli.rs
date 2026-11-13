use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureStorageAccountGetRequest;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// Show one storage account's typed properties directly through Azure Resource Manager.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageAccountShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Full ARM resource ID of the storage account.
    #[facet(figue::named, proxy = String)]
    pub storage_account: AzureStorageAccountId,
    /// Refresh only this storage account's cached properties.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureStorageAccountShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let account = AzureStorageAccountGetRequest {
            authentication: self.authentication.resolve(executor).await?,
            storage_account_id: Cow::Borrowed(&self.storage_account),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(account))
    }
}
