use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_plans;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps test plan details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanShowArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Test plan id or name.
    #[facet(figue::named)]
    pub plan: String,
}

impl AzureDevOpsTestPlanShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let plans = fetch_azure_devops_test_plans(&org_url, self.project).await?;
        if let Some(plan) = plans
            .into_iter()
            .find(|p| p.name == self.plan || p.id.to_string() == self.plan)
        {
            to_writer_pretty(stdout(), &plan)?;
            Ok(())
        } else {
            bail!("No test plan found matching '{}'.", self.plan);
        }
    }
}
