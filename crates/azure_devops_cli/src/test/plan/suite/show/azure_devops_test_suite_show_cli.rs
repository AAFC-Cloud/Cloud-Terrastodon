use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_suites;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test suite details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteShowArgs {
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

    /// Suite id or name.
    #[facet(figue::named)]
    pub suite: String,
}

impl AzureDevOpsTestSuiteShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let suites = fetch_azure_devops_test_suites(
            &self.org,
            self.project,
            self.plan,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;
        if let Some(suite) = suites
            .into_iter()
            .find(|s| s.name == self.suite || s.id.to_string() == self.suite)
        {
            facet_json::to_writer_std_pretty(stdout(), &suite)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No test suite found matching '{}'.", self.suite);
        }
    }
}
