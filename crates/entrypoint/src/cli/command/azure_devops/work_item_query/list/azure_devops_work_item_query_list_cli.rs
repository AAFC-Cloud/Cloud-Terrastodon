use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_queries_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps work item queries for a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryListArgs {
    /// Project id or project name.
    #[facet(figue::named, opaque, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsWorkItemQueryListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let project_name: AzureDevOpsProjectName = match self.project {
            AzureDevOpsProjectArgument::Name(n) => n.into_owned(),
            _ => {
                let projects = fetch_all_azure_devops_projects(&org_url).await?;
                if let Some(project) = projects.into_iter().find(|p| self.project.matches(p)) {
                    project.name
                } else {
                    eyre::bail!("No project found matching '{}'.", self.project);
                }
            }
        };
        let queries = fetch_queries_for_project(&org_url, &project_name).await?;
        to_writer_pretty(stdout(), &queries)?;
        Ok(())
    }
}
