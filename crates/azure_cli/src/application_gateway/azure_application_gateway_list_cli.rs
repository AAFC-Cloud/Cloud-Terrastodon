use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureApplicationGatewayListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure application gateways.
#[derive(Facet, Debug, Clone)]
pub struct AzureApplicationGatewayListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureApplicationGatewayListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Azure application gateways");
        let application_gateways = AzureApplicationGatewayListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = application_gateways.len(),
            "Fetched Azure application gateways"
        );

        Ok(CliOutput::facet(application_gateways))
    }
}
