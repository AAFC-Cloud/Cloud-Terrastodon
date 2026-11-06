use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure role definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleDefinitionBrowseArgs {
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
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(|invalidate| {
                let tenant_auth_context = tenant_auth_context.clone();
                async move {
                    info!("Fetching Azure role definitions");
                    let role_definitions =
                        fetch_all_role_definitions(&tenant_auth_context, executor)
                            .with_invalidation(invalidate)
                            .await?;
                    info!(
                        count = role_definitions.len(),
                        "Fetched Azure role definitions"
                    );
                    Ok(role_definitions)
                }
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
