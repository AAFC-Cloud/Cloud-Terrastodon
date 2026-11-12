use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::discover_and_track_tenants;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Discover tracked Azure tenants from Azure CLI accounts.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantDiscoverArgs {}

impl AzureTenantDiscoverArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenants = discover_and_track_tenants(cancellation).await?;

        Ok(CliOutput::facet(tenants))
    }
}
