use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemFieldShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Field reference name.
    #[facet(figue::positional)]
    pub field: AzureDevOpsWorkItemFieldName,
}

impl AzureDevOpsWorkItemFieldShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let item = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Owned(self.org),
            project: Some(self.project),
            auth_selection,
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
        Ok(CliOutput::facet(value))
    }
}
