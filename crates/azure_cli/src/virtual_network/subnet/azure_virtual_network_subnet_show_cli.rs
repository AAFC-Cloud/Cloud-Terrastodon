use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureSubnetArgument;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::AzureVirtualNetworkListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Show one subnet's modeled prefixes, policies, delegations, and resource references.
#[derive(Debug, Clone, Facet)]
pub struct AzureVirtualNetworkSubnetShowArgs {
    /// Subnet name or full Azure resource ID, matched without case sensitivity.
    /// Must match exactly one subnet within the selected virtual networks.
    #[facet(figue::positional, figue::label = "SUBNET")]
    pub subnet: AzureSubnetArgument,
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Restrict the search to matching virtual networks by name or full resource ID.
    #[facet(figue::named)]
    pub vnet: Option<AzureVirtualNetworkArgument>,
    /// Refresh the tenant's virtual-network inventory before matching subnets.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureVirtualNetworkSubnetShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let networks = AzureVirtualNetworkListRequest { authentication }
            .execute(executor)
            .with_invalidation(self.no_cache)
            .await?;
        let mut subnets = networks
            .iter()
            .filter(|network| {
                self.vnet
                    .as_ref()
                    .is_none_or(|vnet| vnet.matches_id(&network.id))
            })
            .flat_map(|network| network.properties.subnets.iter())
            .filter(|subnet| self.subnet.matches_id(&subnet.id))
            .collect::<Vec<_>>();
        match subnets.len() {
            0 => {
                bail!(
                    "No subnet found matching '{}' in the selected virtual networks.",
                    self.subnet
                );
            }
            1 => Ok(CliOutput::facet(subnets.remove(0).clone())),
            count => {
                subnets.sort_by_key(|subnet| subnet.id.expanded_form());
                let ids = subnets
                    .iter()
                    .map(|subnet| subnet.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "{count} subnets matched '{}'. Use --vnet or a full subnet resource ID.\n  {ids}",
                    self.subnet
                );
            }
        }
    }
}
