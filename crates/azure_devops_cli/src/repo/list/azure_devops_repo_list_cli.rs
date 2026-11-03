use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps repositories in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsRepoListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsRepoListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context, cancellation)
                .await?;

        let Some(project) = projects
            .into_iter()
            .find(|project| self.project.matches(project))
        else {
            eyre::bail!("No project found matching '{}'.", self.project);
        };
        let repos =
            fetch_all_azure_devops_repos_for_project(&self.org, &project.id, cancellation).await?;
        to_writer_pretty(stdout(), &repos)?;
        Ok(())
    }
}
