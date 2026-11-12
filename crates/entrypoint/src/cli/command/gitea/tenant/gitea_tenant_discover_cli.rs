use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::discover_and_track_tenants;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantDiscoverArgs {}

impl GiteaTenantDiscoverArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenants = discover_and_track_tenants(executor).await?;
        Ok(CliOutput::facet(tenants))
    }
}
