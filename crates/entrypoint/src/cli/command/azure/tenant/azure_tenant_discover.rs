use cloud_terrastodon_azure::discover_and_track_tenants;
use eyre::Result;
use std::io::Write;

/// Discover tracked Azure tenants from Azure CLI accounts.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantDiscoverArgs {}

impl AzureTenantDiscoverArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenants = discover_and_track_tenants().await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &tenants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
