use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemType;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemTypeFieldListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemTypeFieldListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Work item type name.
    #[facet(figue::named, rename = "type")]
    pub work_item_type: AzureDevOpsWorkItemType,
}

impl AzureDevOpsWorkItemTypeFieldListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let values = AzureDevOpsWorkItemTypeFieldListRequest {
            org_url: Cow::Owned(self.org),
            project: self.project,
            auth_selection,
            work_item_type: self.work_item_type,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(values))
    }
}
