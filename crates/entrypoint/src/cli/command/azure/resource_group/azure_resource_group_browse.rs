use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resource_groups;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resource groups.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceGroupBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceGroupBrowseArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        browse_resource_groups(&auth_context).await
    }
}

pub async fn browse_resource_groups(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let chosen = PickerTui::<_>::new()
        .pick_many_reloadable(|invalidate| {
            let auth_context = auth_context.clone();
            async move {
                info!(tenant_id = %auth_context.tenant_id, "Fetching all Azure resource groups");

                fetch_all_resource_groups(&auth_context)
                    .with_invalidation(invalidate)
                    .await
            }
        })
        .await?;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
    handle.write_all(b"\n")?;
    Ok(())
}
