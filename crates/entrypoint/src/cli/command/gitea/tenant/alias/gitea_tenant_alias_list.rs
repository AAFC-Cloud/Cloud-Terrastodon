use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases_for;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantAliasListArgs {
    /// Optional tracked tenant URL or alias to filter by.
    #[arg(long)]
    pub tenant: Option<GiteaTenantArgument<'static>>,
}

impl GiteaTenantAliasListArgs {
    pub async fn invoke(self) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        if let Some(tenant) = self.tenant {
            let tenant_url = tenant.resolve().await?;
            let mut aliases = list_tracked_tenant_aliases_for(&tenant_url).await?;
            aliases.sort();
            cloud_terrastodon_command::to_writer_pretty(&mut handle, &aliases)?;
        } else {
            let aliases = list_tracked_tenant_aliases().await?;
            cloud_terrastodon_command::to_writer_pretty(&mut handle, &aliases)?;
        }

        handle.write_all(b"\n")?;
        Ok(())
    }
}
