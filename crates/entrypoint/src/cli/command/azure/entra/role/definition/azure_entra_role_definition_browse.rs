use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_entra_role_definitions;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Entra role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraRoleDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraRoleDefinitionBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .set_header("Entra role definitions")
            .pick_many_reloadable(async |invalidate| {
                info!(%tenant_id, "Fetching Entra role definitions");
                let mut role_definitions = fetch_all_entra_role_definitions(tenant_id)
                    .with_invalidation(invalidate)
                    .await?
                    .into_iter()
                    .collect::<Vec<_>>();
                role_definitions.sort_unstable_by(|left, right| {
                    left.display_name
                        .cmp(&right.display_name)
                        .then_with(|| left.template_id.to_string().cmp(&right.template_id.to_string()))
                });
                let choices = role_definitions.into_iter().map(|definition| Choice {
                    key: format!(
                        "{}\nrole definition id: {}\nbuilt in: {}\nprivileged: {}\nenabled: {}",
                        definition.display_name,
                        definition.template_id,
                        definition.is_built_in,
                        definition.is_privileged,
                        definition.is_enabled
                    ),
                    value: definition,
                });
                Ok(choices)
            })
            .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
