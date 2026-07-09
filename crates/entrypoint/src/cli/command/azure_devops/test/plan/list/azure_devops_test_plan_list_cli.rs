use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_test_plans;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps test plans in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanListArgs {
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsTestPlanListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let plans = fetch_azure_devops_test_plans(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &plans)?;
        Ok(())
    }
}
