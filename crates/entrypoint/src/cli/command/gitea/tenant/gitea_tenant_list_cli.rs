use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_gitea::list_tracked_tenants;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantListArgs {}

impl GiteaTenantListArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenants = list_tracked_tenants().await?;
        Ok(CliOutput::facet(tenants))
    }
}
