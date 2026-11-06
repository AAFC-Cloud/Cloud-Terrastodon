use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps repo details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsRepoShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        // Find a project matching the provided identifier (id or name).
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context, executor)
                .await?;
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            let repos = fetch_all_azure_devops_repos_for_project(
                &self.org,
                &project.id,
                &azure_devops_auth_context,
                executor,
            )
            .await?;
            if let Some(repo) = repos
                .iter()
                .find(|r| r.name == self.repo || r.id.to_string() == self.repo)
            {
                facet_json::to_writer_std_pretty(stdout(), &repo)
                    .map_err(|error| eyre::eyre!("{error:?}"))?;
                Ok(())
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
