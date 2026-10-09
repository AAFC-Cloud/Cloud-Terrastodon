use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_virtual_networks;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// List modeled subnets from the tenant's cached virtual-network inventory.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkSubnetListArgs {
    /// Tenant ID or tracked alias. Defaults to the selected authentication tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let networks = fetch_all_virtual_networks(&tenant_auth_context)
            .with_invalidation(self.no_cache)
            .await?;
        let mut subnets = networks
            .into_iter()
            .filter(|network| {
                self.vnet
                    .as_ref()
                    .is_none_or(|vnet| vnet.matches_id(&network.id))
            })
            .flat_map(|network| network.properties.subnets)
            .collect::<Vec<_>>();
        subnets.sort_by_key(|subnet| subnet.id.expanded_form());
        Ok(CliOutput::facet(subnets))
    }
}
