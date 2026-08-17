use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps test cases in a suite.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
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
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let cases =
            fetch_azure_devops_test_suite_cases(&org_url, self.project, self.plan, self.suite)
                .await?;
        to_writer_pretty(stdout(), &cases)?;
        Ok(())
    }
}
