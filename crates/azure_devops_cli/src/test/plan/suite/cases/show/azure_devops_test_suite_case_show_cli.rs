use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTestSuiteCasesListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps test case details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseShowArgs {
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

    /// Test case id or name.
    #[facet(figue::named)]
    pub case: String,
}

impl AzureDevOpsTestSuiteCaseShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let case_id = self.case.parse::<AzureDevOpsWorkItemId>().ok();
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
        if let Some(case) = cases.iter().find(|c| {
            c.test_case
                .name
                .as_ref()
                .map(|n| n == &self.case)
                .unwrap_or(false)
                || case_id.is_some_and(|id| c.test_case.id == Some(id))
        }) {
            Ok(CliOutput::facet(case.clone()))
        } else {
            bail!("No test case found matching '{}'.", self.case);
        }
    }
}
