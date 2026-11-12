use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsRepositoriesListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

/// Show Azure DevOps repo details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsRepoShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Repository id (UUID) or repository name.
    #[facet(figue::named)]
    pub repo: String,
}

impl AzureDevOpsRepoShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        // Find a project matching the provided identifier (id or name).
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        let maybe = projects.iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            let repos = AzureDevOpsRepositoriesListRequest {
                org_url: Cow::Borrowed(&self.org),
                project_id: project.id,
                auth_selection,
            }
            .execute(executor)
            .await?;
            if let Some(repo) = repos
                .iter()
                .find(|r| r.name == self.repo || r.id.to_string() == self.repo)
            {
                Ok(CliOutput::facet(repo.clone()))
            } else {
                bail!(
                    "No repository found matching '{}' in project {}.",
                    self.repo,
                    project.name
                );
            }
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
    }
}
