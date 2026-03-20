use clap::Args;
use cloud_terrastodon_azure::prelude::discover_and_track_tenants;
use eyre::Result;
use std::io::Write;

/// Discover tracked Azure tenants from Azure CLI accounts.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantDiscoverArgs {}

impl AzureTenantDiscoverArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenants = discover_and_track_tenants().await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &tenants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
