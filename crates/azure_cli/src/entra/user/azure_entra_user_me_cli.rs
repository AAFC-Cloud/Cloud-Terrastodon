use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// Show the signed-in Entra user for the selected account.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserMeArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraUserMeArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        eyre::ensure!(
            authentication.auth_source != AzureBearerAuthSource::WorkloadIdentity,
            "the selected source has no signed-in Entra user"
        );
        let user = EntraCurrentUserRequest { authentication }
            .execute(executor)
            .with_invalidation(self.no_cache)
            .await?;

        Ok(CliOutput::facet(user))
    }
}
