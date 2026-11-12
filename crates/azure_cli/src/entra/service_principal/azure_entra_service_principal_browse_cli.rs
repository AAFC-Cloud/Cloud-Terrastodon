use crate::browse_service_principals;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Interactively browse Entra (Azure AD) service principals.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraSpBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraSpBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        browse_service_principals(authentication, executor).await
    }
}
