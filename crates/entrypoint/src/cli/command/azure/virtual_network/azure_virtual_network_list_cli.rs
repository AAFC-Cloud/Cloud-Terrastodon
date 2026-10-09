use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_virtual_networks;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// List virtual networks from the tenant's cached Azure Resource Graph inventory.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureVirtualNetworkListArgs {
    /// Tenant ID or tracked alias. Defaults to the selected authentication tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
    /// Refresh the tenant's virtual-network inventory before listing it.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
}

impl AzureVirtualNetworkListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let mut networks = fetch_all_virtual_networks(&tenant_auth_context)
            .with_invalidation(self.no_cache)
            .await?;
        networks.sort_by_cached_key(|network| {
            let id = network.id.expanded_form();
            (id.to_ascii_lowercase(), id)
        });
        Ok(CliOutput::facet(networks))
    }
}
