use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_azure_tenant_details;
use eyre::Result;
use std::io::Write;

/// Arguments for showing a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantShowArgs {
    /// Tenant id (GUID) or alias to show.
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let details = fetch_azure_tenant_details(tenant_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &details)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
