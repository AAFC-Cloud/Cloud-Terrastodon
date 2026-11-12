use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases_for;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantAliasListArgs {
    /// Optional tracked tenant URL or alias to filter by.
    #[facet(figue::named)]
    pub tenant: Option<GiteaTenantArgument<'static>>,
}

impl GiteaTenantAliasListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        if let Some(tenant) = self.tenant {
            let tenant_url = tenant.resolve(executor.cancellation()).await?;
            let mut aliases = list_tracked_tenant_aliases_for(&tenant_url).await?;
            aliases.sort();
            Ok(CliOutput::facet(aliases))
        } else {
            let aliases = list_tracked_tenant_aliases().await?;
            Ok(CliOutput::facet(aliases))
        }
    }
}
