use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_operation_metadata;
use cloud_terrastodon_azure::flatten_role_operations;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure provider operations.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleOperationBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleOperationBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        let tenant_id = tenant_auth_context.tenant_id;
        let auth_source = tenant_auth_context.auth_context.azure_bearer_source()?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(|invalidate| async move {
                info!(%tenant_id, "Fetching Azure provider operations metadata");
                let provider_operations =
                    fetch_all_role_operation_metadata(tenant_id, auth_source, executor)
                        .with_invalidation(invalidate)
                        .await?;
                let mut operations = flatten_role_operations(provider_operations);
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
            .await
            .into_chosen_and_maybe_error()?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &chosen)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}
