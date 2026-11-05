use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_plans;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps test plans in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsTestPlanListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let plans = fetch_azure_devops_test_plans(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;
        facet_json::to_writer_std_pretty(stdout(), &plans)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
