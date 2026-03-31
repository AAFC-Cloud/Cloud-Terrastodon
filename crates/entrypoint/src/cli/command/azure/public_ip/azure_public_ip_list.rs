use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_public_ips;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure public IP addresses.
#[derive(Args, Debug, Clone)]
pub struct AzurePublicIpListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePublicIpListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure public IP addresses");
        let public_ips = fetch_all_public_ips(tenant_id).await?;
        info!(
            count = public_ips.len(),
            "Fetched Azure public IP addresses"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &public_ips)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
