use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test case details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseShowArgs {
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

    /// Test case id or name.
    #[facet(figue::named)]
    pub case: String,
}

impl AzureDevOpsTestSuiteCaseShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let case_id = self.case.parse::<AzureDevOpsWorkItemId>().ok();
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let cases = fetch_azure_devops_test_suite_cases(
            &org_url,
            self.project,
            self.plan,
            self.suite,
            &azure_devops_auth_context,
        )
        .await?;
        if let Some(case) = cases.into_iter().find(|c| {
            c.test_case
                .name
                .as_ref()
                .map(|n| n == &self.case)
                .unwrap_or(false)
                || case_id.is_some_and(|id| c.test_case.id == Some(id))
        }) {
            to_writer_pretty(stdout(), &case)?;
            Ok(())
        } else {
            bail!("No test case found matching '{}'.", self.case);
        }
    }
}
