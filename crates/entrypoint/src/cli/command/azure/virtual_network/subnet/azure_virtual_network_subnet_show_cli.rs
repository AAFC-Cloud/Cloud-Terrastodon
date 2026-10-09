use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureSubnetArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_virtual_networks;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;

/// Show one subnet's modeled prefixes, policies, delegations, and resource references.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkSubnetShowArgs {
    /// Subnet name or full Azure resource ID, matched without case sensitivity.
    /// Must match exactly one subnet within the selected virtual networks.
    #[facet(figue::positional, figue::label = "SUBNET")]
    pub subnet: AzureSubnetArgument,
    /// Tenant ID or tracked alias. Defaults to the selected authentication tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
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
            .filter(|subnet| self.subnet.matches_id(&subnet.id))
            .collect::<Vec<_>>();
        match subnets.len() {
            0 => bail!(
                "No subnet found matching '{}' in the selected virtual networks.",
                self.subnet
            ),
            1 => Ok(CliOutput::facet(subnets.remove(0))),
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
                )
            }
        }
    }
}
