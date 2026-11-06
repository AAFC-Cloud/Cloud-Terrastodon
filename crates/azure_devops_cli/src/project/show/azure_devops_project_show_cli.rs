use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id (UUID) or project name.
    #[facet(figue::positional, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context, executor)
                .await?;

        // Find the project matching the resolved ID or name.
        let maybe = projects.into_iter().find(|p| self.project.matches(p));

        if let Some(project) = maybe {
            facet_json::to_writer_std_pretty(stdout(), &project)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
        } else {
            bail!("No project found matching '{}'.", self.project);
        }
        Ok(())
    }
}
