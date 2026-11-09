use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resource groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceGroupBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceGroupBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &auth_context.auth_context.bind_executor(executor)?;
        browse_resource_groups(&auth_context, executor).await
    }
}

pub async fn browse_resource_groups(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!(tenant_id = %auth_context.tenant_id, "Fetching all Azure resource groups");

            fetch_all_resource_groups(auth_context, executor)
                .with_invalidation(invalidate)
                .await
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
