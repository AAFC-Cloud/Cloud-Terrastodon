use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Azure DevOps project-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id (UUID) or project name.
    #[facet(figue::positional, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;

        // Find the project matching the resolved ID or name.
        let maybe = projects.iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            Ok(CliOutput::facet(project.clone()))
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
    }
}
