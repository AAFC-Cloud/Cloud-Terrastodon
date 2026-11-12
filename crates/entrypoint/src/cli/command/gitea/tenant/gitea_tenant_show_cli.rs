use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::get_tracked_tenant;
use eyre::Result;
use eyre::bail;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantShowArgs {
    /// Tracked tenant URL or alias to show.
    #[facet(figue::positional, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaTenantShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant_url = self.tenant.resolve(executor.cancellation()).await?;
        let Some(tenant) = get_tracked_tenant(&tenant_url).await? else {
            bail!("Tracked Gitea tenant '{}' was not found.", tenant_url);
        };
        Ok(CliOutput::facet(tenant))
    }
}
