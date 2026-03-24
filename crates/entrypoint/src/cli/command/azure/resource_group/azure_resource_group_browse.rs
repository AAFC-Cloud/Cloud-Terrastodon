use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceGroupBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceGroupBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                info!(%tenant_id, "Fetching all Azure resource groups");

                fetch_all_resource_groups(tenant_id)
                    .with_invalidation(invalidate)
                    .await
            })
            .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
