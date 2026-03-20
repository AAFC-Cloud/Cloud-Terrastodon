use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::list_tracked_tenant_aliases;
use cloud_terrastodon_azure::prelude::list_tracked_tenant_aliases_for;
use eyre::Result;
use std::io::Write;

/// Arguments for listing aliases for tracked Azure tenants.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAliasListArgs {
    /// Optional tracked tenant id or Cloud Terrastodon alias to filter by.
    #[arg(long)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureTenantAliasListArgs {
    pub async fn invoke(self) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        if let Some(tenant) = self.tenant {
            let tenant_id = tenant.resolve().await?;
            let mut aliases = list_tracked_tenant_aliases_for(tenant_id).await?;
            aliases.sort();
            serde_json::to_writer_pretty(&mut handle, &aliases)?;
        } else {
            let aliases = list_tracked_tenant_aliases().await?;
            serde_json::to_writer_pretty(&mut handle, &aliases)?;
        }

        handle.write_all(b"\n")?;
        Ok(())
    }
}
