use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemQueryListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Folder expansion depth, from zero to two.
    #[facet(figue::named, default = 2)]
    pub depth: u32,
}

impl AzureDevOpsWorkItemQueryListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let queries = AzureDevOpsWorkItemQueryListRequest {
            org_url: Cow::Owned(self.org),
            project: self.project,
            auth_selection,
            depth: self.depth,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(queries))
    }
}
