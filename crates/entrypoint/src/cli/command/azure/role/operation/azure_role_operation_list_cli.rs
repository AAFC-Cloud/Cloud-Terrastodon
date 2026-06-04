use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_operation_metadata;
use cloud_terrastodon_azure::flatten_role_operations;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure provider operations.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleOperationListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleOperationListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure provider operations metadata");
        let provider_operations = fetch_all_role_operation_metadata(tenant_id).await?;
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

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &operations)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
