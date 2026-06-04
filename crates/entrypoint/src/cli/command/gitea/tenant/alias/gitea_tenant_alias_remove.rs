use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantAlias;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::remove_tracked_tenant_aliases;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantAliasRemoveArgs {
    /// Tracked tenant URL or alias.
    #[arg(long)]
    pub tenant: GiteaTenantArgument<'static>,

    /// One or more aliases to remove.
    #[arg(required = true, num_args = 1..)]
    pub aliases: Vec<GiteaTenantAlias>,
}

impl GiteaTenantAliasRemoveArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_url = self.tenant.resolve().await?;
        let aliases = remove_tracked_tenant_aliases(&tenant_url, &self.aliases).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &aliases)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
