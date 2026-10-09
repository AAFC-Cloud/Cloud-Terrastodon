use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps teams in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTeamListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsTeamListArgs {
    pub async fn invoke(self) -> Result<()> {
        let teams = fetch_azure_devops_teams_for_project(&self.org, self.project).await?;
        to_writer_pretty(stdout(), &teams)?;
        Ok(())
    }
}
