use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::AzureVirtualNetworkListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Show one virtual network, including its modeled address spaces, subnets, and peerings.
#[derive(Debug, Clone, Facet)]
pub struct AzureVirtualNetworkShowArgs {
    /// Virtual-network name or full Azure resource ID, matched without case sensitivity.
    /// Must match exactly one virtual network.
    #[facet(figue::positional, figue::label = "VNET")]
    pub virtual_network: AzureVirtualNetworkArgument,
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Refresh the tenant's virtual-network inventory before matching networks.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureVirtualNetworkShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let networks = AzureVirtualNetworkListRequest { authentication }
            .execute(executor)
            .with_invalidation(self.no_cache)
            .await?;
        let mut networks = networks
            .into_iter()
            .filter(|network| self.virtual_network.matches_id(&network.id))
            .collect::<Vec<_>>();
        match networks.len() {
            0 => {
                bail!(
                    "No virtual network found matching '{}'.",
                    self.virtual_network
                );
            }
            1 => Ok(CliOutput::facet(networks.remove(0))),
            count => {
                networks.sort_by_cached_key(|network| {
                    let id = network.id.expanded_form();
                    (id.to_ascii_lowercase(), id)
                });
                let ids = networks
                    .iter()
                    .map(|network| network.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "{count} virtual networks matched '{}'. Use a full resource ID.\n  {ids}",
                    self.virtual_network
                );
            }
        }
    }
}
