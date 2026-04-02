use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_private_endpoints;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure private endpoints.
#[derive(Args, Debug, Clone)]
pub struct AzurePrivateEndpointListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePrivateEndpointListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure private endpoints");
        let private_endpoints = fetch_all_private_endpoints(tenant_id).await?;
        info!(
            count = private_endpoints.len(),
            "Fetched Azure private endpoints"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &private_endpoints)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
