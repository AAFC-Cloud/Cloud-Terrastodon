use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_plans;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::stdout;

/// Show Azure DevOps test plan details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id or name.
    #[facet(figue::named)]
    pub plan: String,
}

impl AzureDevOpsTestPlanShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let plans = fetch_azure_devops_test_plans(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        if let Some(plan) = plans
            .into_iter()
            .find(|p| p.name == self.plan || p.id.to_string() == self.plan)
        {
            facet_json::to_writer_std_pretty(stdout(), &plan)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No test plan found matching '{}'.", self.plan);
        }
    }
}
