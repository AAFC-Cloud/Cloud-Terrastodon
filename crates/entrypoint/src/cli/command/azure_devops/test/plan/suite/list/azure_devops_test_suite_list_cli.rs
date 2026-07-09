use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suites;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps test suites in a plan.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteListArgs {
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id.
    #[facet(figue::named)]
    pub plan: String,
}

impl AzureDevOpsTestSuiteListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let suites = fetch_azure_devops_test_suites(&org_url, self.project, self.plan).await?;
        to_writer_pretty(stdout(), &suites)?;
        Ok(())
    }
}
