use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::forget_tracked_tenant;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantForgetArgs {
    /// Tracked tenant URL or alias to forget.
    #[facet(figue::positional, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaTenantForgetArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant_url = self.tenant.resolve(executor.cancellation()).await?;
        let forgotten = forget_tracked_tenant(&tenant_url).await?;
        Ok(CliOutput::facet(forgotten))
    }
}
