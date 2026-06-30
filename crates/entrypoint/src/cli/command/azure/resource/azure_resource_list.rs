use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resources;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure resources.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching all Azure resources");
        let resources = fetch_all_resources(tenant_id).await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &resources)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
