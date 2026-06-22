use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_operation_metadata;
use cloud_terrastodon_azure::flatten_role_operations;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure provider operations.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleOperationBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleOperationBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                info!(%tenant_id, "Fetching Azure provider operations metadata");
                let provider_operations = fetch_all_role_operation_metadata(tenant_id)
                    .with_invalidation(invalidate)
                    .await?;
                let mut operations = flatten_role_operations(&provider_operations);
                operations.sort_by(|left, right| {
                    left.name
                        .to_string()
                        .cmp(&right.name.to_string())
                        .then(left.provider_name.cmp(&right.provider_name))
                        .then(left.resource_type_name.cmp(&right.resource_type_name))
                });
                info!(
                    count = operations.len(),
                    "Fetched Azure provider operations"
                );
                Ok(operations)
            })
            .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
