use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps groups in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsGroupListArgs {
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let groups = fetch_azure_devops_groups_for_project(&org_url, self.project).await?;
        to_writer_pretty(stdout(), &groups)?;
        Ok(())
    }
}
