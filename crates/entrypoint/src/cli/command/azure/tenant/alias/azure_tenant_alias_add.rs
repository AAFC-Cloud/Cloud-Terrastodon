use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantAlias;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::add_tracked_tenant_aliases;
use eyre::Result;
use std::io::Write;

/// Arguments for adding aliases to a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAliasAddArgs {
    /// Tracked tenant id or Cloud Terrastodon alias.
    #[arg(long)]
    pub tenant: AzureTenantArgument<'static>,

    /// One or more aliases to add to the tracked tenant.
    #[arg(required = true, num_args = 1..)]
    pub aliases: Vec<AzureTenantAlias>,
}

impl AzureTenantAliasAddArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let aliases = add_tracked_tenant_aliases(tenant_id, &self.aliases).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &aliases)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
