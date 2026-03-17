use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::get_tracked_tenant;
use cloud_terrastodon_azure::prelude::resolve_tracked_tenant_argument;
use eyre::Result;
use eyre::bail;

/// Arguments for showing a tracked Azure tenant.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantShowArgs {
    /// Tenant id (GUID) or alias to show.
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = resolve_tracked_tenant_argument(self.tenant).await?;
        let Some(tenant) = get_tracked_tenant(tenant_id.clone()).await? else {
            bail!("Tracked tenant '{}' was not found.", tenant_id);
        };

        println!("{}", tenant.as_hyphenated());
        Ok(())
    }
}
