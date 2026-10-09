use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::AzureVirtualNetworkArgument;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_virtual_networks;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;

/// Show one virtual network, including its modeled address spaces, subnets, and peerings.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkShowArgs {
    /// Virtual-network name or full Azure resource ID, matched without case sensitivity.
    /// Must match exactly one virtual network.
    #[facet(figue::positional, figue::label = "VNET")]
    pub virtual_network: AzureVirtualNetworkArgument,
    /// Tenant ID or tracked alias. Defaults to the selected authentication tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let networks = fetch_all_virtual_networks(&tenant_auth_context)
            .with_invalidation(self.no_cache)
            .await?;
        let mut networks = networks
            .into_iter()
            .filter(|network| self.virtual_network.matches_id(&network.id))
            .collect::<Vec<_>>();
        match networks.len() {
            0 => bail!(
                "No virtual network found matching '{}'.",
                self.virtual_network
            ),
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
                )
            }
        }
    }
}
