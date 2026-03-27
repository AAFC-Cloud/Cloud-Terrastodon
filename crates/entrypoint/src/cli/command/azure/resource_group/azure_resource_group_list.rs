use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceGroupListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching all Azure resource groups");
        let groups = fetch_all_resource_groups(tenant_id).await?;
        info!(count = groups.len(), "Fetched resource groups");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
