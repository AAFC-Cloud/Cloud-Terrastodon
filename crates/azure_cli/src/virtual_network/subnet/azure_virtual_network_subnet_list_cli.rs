use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::AzureVirtualNetworkListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List modeled subnets from the tenant's cached virtual-network inventory.
#[derive(Debug, Clone, Facet)]
pub struct AzureVirtualNetworkSubnetListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Virtual-network name or full resource ID. Include subnets from every matching network.
    /// Omit to list subnets across the tenant.
    #[facet(figue::named)]
    pub vnet: Option<AzureVirtualNetworkArgument>,
    /// Refresh the tenant's virtual-network inventory before listing subnets.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureVirtualNetworkSubnetListArgs {
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
            .collect::<Vec<_>>();
        subnets.sort_by_key(|subnet| subnet.id.expanded_form());
        Ok(CliOutput::facet(
            subnets.into_iter().cloned().collect::<Vec<_>>(),
        ))
    }
}
