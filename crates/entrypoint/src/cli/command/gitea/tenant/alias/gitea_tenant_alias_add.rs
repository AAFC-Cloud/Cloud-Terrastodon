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
    pub async fn invoke(self) -> Result<()> {
        let tenant_url = self.tenant.resolve().await?;
        let aliases = add_tracked_tenant_aliases(&tenant_url, &self.aliases).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &aliases)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
