use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps test cases in a suite.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[facet(figue::named)]
    pub plan: String,

    /// Suite id.
    #[facet(figue::named)]
    pub suite: String,
}

impl AzureDevOpsTestSuiteCaseListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let cases = fetch_azure_devops_test_suite_cases(
            &self.org,
            self.project,
            self.plan,
            self.suite,
            &azure_devops_auth_context,
        )
        .await?;
        to_writer_pretty(stdout(), &cases)?;
        Ok(())
    }
}
