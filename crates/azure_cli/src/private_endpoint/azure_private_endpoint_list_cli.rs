use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrivateEndpointListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure private endpoints.
#[derive(Facet, Debug, Clone)]
pub struct AzurePrivateEndpointListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzurePrivateEndpointListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Azure private endpoints");
        let private_endpoints = AzurePrivateEndpointListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = private_endpoints.len(),
            "Fetched Azure private endpoints"
        );

        Ok(CliOutput::facet(private_endpoints))
    }
}
