use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for browsing Entra role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleDefinitionBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraRoleDefinitionBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .set_header("Entra role definitions")
            .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                let executor = &executor.with_cancellation_scope(child);
                info!(tenant_id = %authentication.tenant_id, "Fetching Entra role definitions");
                let mut role_definitions = EntraUnifiedRoleDefinitionListRequest { authentication }
                    .execute(executor)
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

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
