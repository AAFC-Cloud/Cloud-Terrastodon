use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suite_cases;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test case details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let case_id = self.case.parse::<AzureDevOpsWorkItemId>().ok();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let cases = fetch_azure_devops_test_suite_cases(
            &self.org,
            self.project,
            self.plan,
            self.suite,
            &azure_devops_auth_context,
            cancellation,
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
            facet_json::to_writer_std_pretty(stdout(), &case)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No test case found matching '{}'.", self.case);
        }
    }
}
