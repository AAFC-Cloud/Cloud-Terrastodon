use clap::Args;
use cloud_terrastodon_azure::list_tracked_tenants;
use eyre::Result;
use std::io::Write;

/// Arguments for listing tracked Azure tenants.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantListArgs {}

impl AzureTenantListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenants = list_tracked_tenants().await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &tenants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
