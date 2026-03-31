use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_application_gateways;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure application gateways.
#[derive(Args, Debug, Clone)]
pub struct AzureApplicationGatewayListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureApplicationGatewayListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure application gateways");
        let application_gateways = fetch_all_application_gateways(tenant_id).await?;
        info!(
            count = application_gateways.len(),
            "Fetched Azure application gateways"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &application_gateways)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
