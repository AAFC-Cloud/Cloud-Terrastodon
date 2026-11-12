use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePublicIpListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for listing Azure public IP addresses.
#[derive(Facet, Debug, Clone)]
pub struct AzurePublicIpListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzurePublicIpListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Azure public IP addresses");
        let public_ips = AzurePublicIpListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = public_ips.len(),
            "Fetched Azure public IP addresses"
        );

        Ok(CliOutput::facet(public_ips))
    }
}
