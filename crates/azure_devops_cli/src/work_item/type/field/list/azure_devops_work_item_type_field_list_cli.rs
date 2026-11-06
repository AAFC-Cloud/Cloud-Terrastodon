use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemType;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemTypeFieldListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemTypeFieldListArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item type name.
    #[facet(figue::named, rename = "type")]
    pub work_item_type: AzureDevOpsWorkItemType,
}

impl AzureDevOpsWorkItemTypeFieldListArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self.tenant.bind_auth_context(auth, cancellation).await?;
        let executor = &auth_context.bind_executor(executor)?;
        let values = AzureDevOpsWorkItemTypeFieldListRequest {
            org_url: Cow::Owned(self.org),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            work_item_type: self.work_item_type,
        }
        .execute(executor)
        .await?;
        facet_json::to_writer_std_pretty(std::io::stdout(), &values)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
