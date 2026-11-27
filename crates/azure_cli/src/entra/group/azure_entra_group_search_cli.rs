use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupSearchRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// Search Entra groups by object ID or display-name tokens.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraGroupSearchArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Group object ID or display-name search text.
    #[facet(figue::positional)]
    pub search_term: String,

    /// Fetch fresh data from Microsoft Graph.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureEntraGroupSearchArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let groups = EntraGroupSearchRequest {
            authentication,
            search_term: self.search_term,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;

        Ok(CliOutput::facet(groups))
    }
}
