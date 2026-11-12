use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTestSuiteCasesListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List Azure DevOps test cases in a suite.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseListArgs {
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

    /// Suite id.
    #[facet(figue::named)]
    pub suite: String,
}

impl AzureDevOpsTestSuiteCaseListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let cases = AzureDevOpsTestSuiteCasesListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            plan: (self.plan),
            suite: (self.suite),
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(cases))
    }
}
