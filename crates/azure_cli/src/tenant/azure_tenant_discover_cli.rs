use cloud_terrastodon_azure::discover_and_track_tenants;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::Write;

/// Discover tracked Azure tenants from Azure CLI accounts.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantDiscoverArgs {}

impl AzureTenantDiscoverArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        let tenants = discover_and_track_tenants(cancellation).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &tenants)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
