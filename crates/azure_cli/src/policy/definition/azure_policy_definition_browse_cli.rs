use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicyDefinitionListRequest;
use cloud_terrastodon_azure::PolicyDefinition;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Azure policy definitions interactively.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyDefinitionBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzurePolicyDefinitionBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure policy definitions...");
        let policy_definitions = AzurePolicyDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions",
        );

        let choices = policy_definitions.into_iter().map(|definition| Choice {
            key: match definition.description.as_ref() {
                Some(description) => format!("{definition} - {description}"),
                None => format!("{definition} - no description"),
            },
            value: definition,
        });

        let (chosen, maybe_error): (Vec<Arc<PolicyDefinition>>, _) = PickerTui::<_>::new()
            .set_header("Select Azure policy definitions")
            .pick_many(choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
