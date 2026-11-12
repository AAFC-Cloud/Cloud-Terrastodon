use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTestSuiteListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps test suite details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[facet(figue::named)]
    pub plan: String,

    /// Suite id or name.
    #[facet(figue::named)]
    pub suite: String,
}

impl AzureDevOpsTestSuiteShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let suites = AzureDevOpsTestSuiteListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            plan: (self.plan),
            auth_selection,
        }
        .execute(executor)
        .await?;
        if let Some(suite) = suites
            .into_iter()
            .find(|s| s.name == self.suite || s.id.to_string() == self.suite)
        {
            Ok(CliOutput::facet(suite))
        } else {
            bail!("No test suite found matching '{}'.", self.suite);
        }
    }
}
