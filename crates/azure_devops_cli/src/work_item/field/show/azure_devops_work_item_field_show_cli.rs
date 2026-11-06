use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemFieldShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Field reference name.
    #[facet(figue::positional)]
    pub field: AzureDevOpsWorkItemFieldName,
}

impl AzureDevOpsWorkItemFieldShowArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self.tenant.bind_auth_context(auth, cancellation).await?;
        let executor = &auth_context.bind_executor(executor)?;
        let item = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Owned(self.org),
            project: Some(self.project),
            auth_context: Cow::Owned(auth_context),
            id: self.id,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .execute(executor)
        .await?;
        let value = item
            .fields
            .field(self.field.as_ref())?
            .ok_or_else(|| eyre::eyre!("Field is not present on the work item"))?;
        facet_json::to_writer_std_pretty(std::io::stdout(), &value)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
