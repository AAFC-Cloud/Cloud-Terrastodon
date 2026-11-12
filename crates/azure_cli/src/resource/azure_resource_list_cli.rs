use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure resources.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureResourceListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching all Azure resources");
        let resources = AzureResourceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = resources.len(), "Fetched Azure resources");

        Ok(CliOutput::facet(resources))
    }
}
