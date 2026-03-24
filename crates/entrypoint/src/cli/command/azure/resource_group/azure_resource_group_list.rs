use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_azure::prelude::get_default_tenant_id;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceGroupListArgs {}

impl AzureResourceGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching all Azure resource groups");
        let tenant_id = get_default_tenant_id().await?;
        let groups = fetch_all_resource_groups(tenant_id).await?;
        info!(count = groups.len(), "Fetched resource groups");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
