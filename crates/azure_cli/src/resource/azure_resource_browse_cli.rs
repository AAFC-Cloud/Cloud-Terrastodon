use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Azure resources interactively.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureResourceBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure resources...");
        let resources = AzureResourceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let choices = resources.into_iter().map(|resource| Choice {
            key: format!("{} - {}", resource.name, resource.id.expanded_form()),
            value: resource,
        });

        let (chosen, maybe_error): (Vec<Arc<Resource>>, _) = PickerTui::<_>::new()
            .set_header("Select Azure resources")
            .pick_many(choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
