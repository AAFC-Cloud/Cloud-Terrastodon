use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleDefinitionBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                info!("Fetching Azure role definitions");
                let role_definitions = fetch_all_role_definitions(tenant_id)
                    .with_invalidation(invalidate)
                    .await?;
                info!(
                    count = role_definitions.len(),
                    "Fetched Azure role definitions"
                );
                Ok(role_definitions)
            })
            .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
