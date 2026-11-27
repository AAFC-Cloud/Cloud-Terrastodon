use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectWorkItemListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::num::NonZeroU16;

/// List the most recently changed work items in a project.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRecentArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Maximum number of work items to return.
    #[facet(figue::named, default = AzureDevOpsProjectWorkItemListRequest::DEFAULT_LIMIT)]
    pub limit: NonZeroU16,
    /// Refresh cached results before listing work items.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureDevOpsWorkItemRecentArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let project_id = match &self.project {
            AzureDevOpsProjectArgument::Id(id) => **id,
            AzureDevOpsProjectArgument::Name(_) => {
                let projects = AzureDevOpsProjectsListRequest {
                    org_url: Cow::Borrowed(&self.org),
                    auth_selection,
                }
                .execute(executor)
                .with_invalidation(self.no_cache)
                .await?;
                let Some(project) = projects
                    .iter()
                    .find(|project| self.project.matches(project))
                else {
                    bail!("No project found matching '{}'.", self.project);
                };
                project.id
            }
        };
        let items = AzureDevOpsProjectWorkItemListRequest {
            org_url: Cow::Owned(self.org),
            project_id,
            auth_selection,
            limit: self.limit,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(items))
    }
}
