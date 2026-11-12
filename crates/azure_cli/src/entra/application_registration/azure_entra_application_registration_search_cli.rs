use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraApplicationRegistrationSearchRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Search Entra (Azure AD) application registrations by app id, name, or URI prefix.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationSearchArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Search text. Graph filters application identifiers and names server-side.
    #[facet(figue::positional)]
    pub search_term: String,
}

impl AzureEntraApplicationRegistrationSearchArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, search_term = %self.search_term, "Searching application registrations");
        let applications = EntraApplicationRegistrationSearchRequest {
            search_term: (self.search_term),
            authentication,
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(applications))
    }
}
