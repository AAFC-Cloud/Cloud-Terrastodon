use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantAlias;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::add_tracked_tenant_aliases;
use eyre::Result;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantAliasAddArgs {
    /// Tracked tenant URL or alias.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,

    /// One or more aliases to add.
    #[facet(figue::positional)]
    pub aliases: Vec<GiteaTenantAlias>,
}

impl GiteaTenantAliasAddArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant_url = self.tenant.resolve(executor).await?;
        let aliases = add_tracked_tenant_aliases(&tenant_url, &self.aliases).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &aliases)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
