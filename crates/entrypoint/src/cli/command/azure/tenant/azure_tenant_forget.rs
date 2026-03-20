use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::forget_tracked_tenant;
use eyre::Result;
use eyre::bail;
use std::io::Write;

/// Arguments for forgetting a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantForgetArgs {
    /// Tenant id (GUID) or alias to forget.
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantForgetArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let Some(tenant) = forget_tracked_tenant(tenant_id).await? else {
            bail!("Tracked tenant '{}' was not found.", tenant_id);
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &tenant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
