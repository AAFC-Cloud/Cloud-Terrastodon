use clap::Args;
use cloud_terrastodon_azure::prelude::TenantId;
use cloud_terrastodon_azure::prelude::add_tracked_tenant;
use eyre::Result;
use std::io::Write;

/// Arguments for adding a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAddArgs {
    /// Tenant id (GUID) to track.
    pub tenant_id: TenantId,
}

impl AzureTenantAddArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = add_tracked_tenant(self.tenant_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &tenant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}