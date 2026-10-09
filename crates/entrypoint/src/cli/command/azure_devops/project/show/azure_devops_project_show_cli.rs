use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectShowArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id (UUID) or project name.
    #[facet(figue::positional, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context).await?;

        // Find the project matching the resolved ID or name.
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            to_writer_pretty(stdout(), &project)?;
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
        Ok(())
    }
}
