use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsRepositoriesListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List Azure DevOps repositories in a project.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsRepoListArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsRepoListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;

        let Some(project) = projects
            .iter()
            .find(|project| self.project.matches(project))
        else {
            eyre::bail!("No project found matching '{}'.", self.project);
        };
        let repos = AzureDevOpsRepositoriesListRequest {
            org_url: Cow::Borrowed(&self.org),
            project_id: project.id,
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(repos))
    }
}
