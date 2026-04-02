use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_app_services;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure App Services.
#[derive(Args, Debug, Clone)]
pub struct AzureAppServiceListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureAppServiceListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching app services");
        let app_services = fetch_all_app_services(tenant_id).await?;
        info!(count = app_services.len(), "Fetched app services");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &app_services)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
