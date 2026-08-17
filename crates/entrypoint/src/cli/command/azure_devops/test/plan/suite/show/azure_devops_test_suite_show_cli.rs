use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suites;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test suite details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteShowArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
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
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let suites = fetch_azure_devops_test_suites(&org_url, self.project, self.plan).await?;
        if let Some(suite) = suites
            .into_iter()
            .find(|s| s.name == self.suite || s.id.to_string() == self.suite)
        {
            to_writer_pretty(stdout(), &suite)?;
            Ok(())
        } else {
            bail!("No test suite found matching '{}'.", self.suite);
        }
    }
}
