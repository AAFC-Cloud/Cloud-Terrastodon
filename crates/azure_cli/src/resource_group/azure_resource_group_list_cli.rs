use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceGroupListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure resource groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceGroupListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureResourceGroupListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching all Azure resource groups");
        let groups = AzureResourceGroupListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = groups.len(), "Fetched resource groups");

        Ok(CliOutput::facet(groups))
    }
}
