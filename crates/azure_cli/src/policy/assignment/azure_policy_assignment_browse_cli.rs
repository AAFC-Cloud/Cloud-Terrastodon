use crate::browse_policy_assignments;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Arguments for browsing Azure policy assignments interactively.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzurePolicyAssignmentBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        browse_policy_assignments(authentication, executor).await
    }
}
