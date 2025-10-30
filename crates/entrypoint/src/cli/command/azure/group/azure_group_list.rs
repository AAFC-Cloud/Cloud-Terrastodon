use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureGroupListArgs {}

impl AzureGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching all Azure resource groups");
        let groups = fetch_all_resource_groups().await?;
        info!(count = groups.len(), "Fetched resource groups");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
