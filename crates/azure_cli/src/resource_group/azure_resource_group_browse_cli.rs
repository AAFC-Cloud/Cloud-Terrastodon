use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceGroupListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Arguments for browsing Azure resource groups.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceGroupBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureResourceGroupBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        browse_resource_groups(authentication, executor).await
    }
}

pub async fn browse_resource_groups(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<CliOutput> {
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
            info!(tenant_id = %authentication.tenant_id, "Fetching all Azure resource groups");

            AzureResourceGroupListRequest { authentication }
                .execute(executor)
                .with_invalidation(invalidate)
                .await
        })
        .await
        .into_chosen_and_maybe_error()?;

    Ok(CliOutput::facet(chosen).with_result(maybe_error))
}
