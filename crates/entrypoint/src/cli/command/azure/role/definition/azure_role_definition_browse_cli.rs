use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_definitions;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(|invalidate| {
                let tenant_auth_context = tenant_auth_context.clone();
                async move {
                    info!("Fetching Azure role definitions");
                    let role_definitions = fetch_all_role_definitions(&tenant_auth_context)
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
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}
