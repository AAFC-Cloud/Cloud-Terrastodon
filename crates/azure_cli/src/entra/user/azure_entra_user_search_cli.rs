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

    /// Search text. Matching is performed against the beginning of user name fields.
    #[facet(figue::positional)]
    pub search_term: String,
}

impl AzureEntraUserSearchArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, search_term = %self.search_term, "Searching Entra users");
        let users = EntraUserSearchRequest {
            search_term: (self.search_term),
            authentication,
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet(users))
    }
}
