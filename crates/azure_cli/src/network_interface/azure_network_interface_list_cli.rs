use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureNetworkInterfaceListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure network interfaces.
#[derive(Facet, Debug, Clone)]
pub struct AzureNetworkInterfaceListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureNetworkInterfaceListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Azure network interfaces");
        let network_interfaces = AzureNetworkInterfaceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = network_interfaces.len(),
            "Fetched Azure network interfaces"
        );

        Ok(CliOutput::facet(network_interfaces))
    }
}
