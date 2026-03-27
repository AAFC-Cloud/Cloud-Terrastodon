use clap::Args;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::remove_tracked_tenant_aliases;
use eyre::Result;
use std::io::Write;

/// Arguments for removing aliases from a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAliasRemoveArgs {
    /// Tracked tenant id or Cloud Terrastodon alias.
    #[arg(long)]
    pub tenant: AzureTenantArgument<'static>,

    /// One or more aliases to remove from the tracked tenant.
    #[arg(required = true, num_args = 1..)]
    pub aliases: Vec<AzureTenantAlias>,
}

impl AzureTenantAliasRemoveArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let aliases = remove_tracked_tenant_aliases(tenant_id, &self.aliases).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &aliases)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
