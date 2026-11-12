use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use facet::Facet;

/// Browse Entra OAuth2 permission grants interactively.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraOAuth2PermissionGrantBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let (chosen, maybe_error) = pick_oauth2_permission_grants(&authentication, executor)
            .await
            .into_chosen_and_maybe_error()?;
        let chosen = chosen
            .into_iter()
            .map(|grant| grant.grant)
            .collect::<Vec<_>>();
        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
