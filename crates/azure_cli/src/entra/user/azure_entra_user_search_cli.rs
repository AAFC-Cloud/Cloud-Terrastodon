use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUserSearchRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Search Entra (Azure AD) users by name, email address, or user principal name.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserSearchArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Search text. The default matches the beginning of user name fields.
    #[facet(figue::positional)]
    pub search_term: String,

    /// Match the full display name or a user object ID.
    #[facet(figue::named, default)]
    pub exact_display_name: bool,

    /// Fetch fresh search results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraUserSearchArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, search_term = %self.search_term, "Searching Entra users");
        let users = EntraUserSearchRequest {
            exact_display_name: self.exact_display_name,
            search_term: (self.search_term),
            authentication,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(users))
    }
}
