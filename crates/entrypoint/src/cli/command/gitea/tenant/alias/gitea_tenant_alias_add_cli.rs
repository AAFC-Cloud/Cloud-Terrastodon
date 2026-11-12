use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantAlias;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::add_tracked_tenant_aliases;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantAliasAddArgs {
    /// Tracked tenant URL or alias.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,

    /// One or more aliases to add.
    #[facet(figue::positional)]
    pub aliases: Vec<GiteaTenantAlias>,
}

impl GiteaTenantAliasAddArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant_url = self.tenant.resolve(executor.cancellation()).await?;
        let aliases = add_tracked_tenant_aliases(&tenant_url, &self.aliases).await?;
        Ok(CliOutput::facet(aliases))
    }
}
