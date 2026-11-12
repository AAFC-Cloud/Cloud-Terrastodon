use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::list_tracked_tenants;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Arguments for listing tracked Azure tenants.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantListArgs {}

impl AzureTenantListArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenants = list_tracked_tenants().await?;

        Ok(CliOutput::facet(tenants))
    }
}
