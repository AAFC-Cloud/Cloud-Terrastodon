use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_groups;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) groups.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Entra groups");
        let groups = fetch_all_groups(tenant_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
