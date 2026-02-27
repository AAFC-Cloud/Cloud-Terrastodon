use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resources.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceListArgs {}

impl AzureResourceListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching all Azure resources");
        let resources = fetch_all_resources().await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &resources)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
