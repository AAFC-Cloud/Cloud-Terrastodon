use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_entra_role_definitions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Entra role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraRoleDefinitionBrowseArgs {
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
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .set_header("Entra role definitions")
            .pick_many_reloadable(async |invalidate| {
                info!(tenant_id = %auth_context.tenant_id, "Fetching Entra role definitions");
                let mut role_definitions =
                    fetch_all_entra_role_definitions(&auth_context, executor)
                        .with_invalidation(invalidate)
                        .await?
                        .into_iter()
                        .collect::<Vec<_>>();
                role_definitions.sort_unstable_by(|left, right| {
                    left.display_name.cmp(&right.display_name).then_with(|| {
                        left.template_id
                            .to_string()
                            .cmp(&right.template_id.to_string())
                    })
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
