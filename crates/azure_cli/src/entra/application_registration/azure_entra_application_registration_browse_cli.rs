use crate::browse_application_registrations;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Interactively browse Entra (Azure AD) application registrations.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraApplicationRegistrationBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        browse_application_registrations(self.authentication.resolve(executor).await?, executor)
            .await
    }
}
