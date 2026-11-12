use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for browsing Azure role definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleDefinitionBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureRoleDefinitionBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                let executor = &executor.with_cancellation_scope(child);
                info!("Fetching Azure role definitions");
                let role_definitions = AzureRoleDefinitionListRequest { authentication }
                    .execute(executor)
                    .with_invalidation(invalidate)
                    .await?;
                info!(
                    count = role_definitions.len(),
                    "Fetched Azure role definitions"
                );
                Ok(role_definitions.to_vec())
            })
            .await
            .into_chosen_and_maybe_error()?;

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
