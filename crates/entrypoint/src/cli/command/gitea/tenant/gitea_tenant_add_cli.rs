use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_gitea::GiteaInstanceUrl;
use cloud_terrastodon_gitea::add_tracked_tenant;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantAddArgs {
    /// Gitea instance URL to track.
    #[facet(figue::positional, proxy = String)]
    pub tenant: GiteaInstanceUrl,
}

impl GiteaTenantAddArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenant = add_tracked_tenant(self.tenant).await?;
        Ok(CliOutput::facet(tenant))
    }
}
