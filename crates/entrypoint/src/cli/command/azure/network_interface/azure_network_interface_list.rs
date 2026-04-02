use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_network_interfaces;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure network interfaces.
#[derive(Args, Debug, Clone)]
pub struct AzureNetworkInterfaceListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureNetworkInterfaceListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure network interfaces");
        let network_interfaces = fetch_all_network_interfaces(tenant_id).await?;
        info!(
            count = network_interfaces.len(),
            "Fetched Azure network interfaces"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &network_interfaces)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
