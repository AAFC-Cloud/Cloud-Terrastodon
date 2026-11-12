use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureVirtualNetworkListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// List virtual networks from the tenant's cached Azure Resource Graph inventory.
#[derive(Debug, Clone, Facet)]
pub struct AzureVirtualNetworkListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let mut networks = AzureVirtualNetworkListRequest { authentication }
            .execute(executor)
            .with_invalidation(self.no_cache)
            .await?;
        networks.sort_by_cached_key(|network| {
            let id = network.id.expanded_form();
            (id.to_ascii_lowercase(), id)
        });
        Ok(CliOutput::facet(networks))
    }
}
