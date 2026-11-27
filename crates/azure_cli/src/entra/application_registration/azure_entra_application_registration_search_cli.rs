use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraApplicationRegistrationSearchRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Search Entra application registrations by object ID, client ID, or display-name tokens.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationSearchArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// An application object or client ID, or display-name search text.
    #[facet(figue::positional)]
    pub search_term: String,

    /// Match the full display name. UUID searches match the object or client ID.
    #[facet(figue::named, default)]
    pub exact_display_name: bool,

    /// Fetch fresh search results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraApplicationRegistrationSearchArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, search_term = %self.search_term, "Searching application registrations");
        let applications = EntraApplicationRegistrationSearchRequest {
            exact_display_name: self.exact_display_name,
            search_term: (self.search_term),
            authentication,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(applications))
    }
}
