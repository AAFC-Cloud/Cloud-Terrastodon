use crate::work_item::r#type::field::list::AzureDevOpsWorkItemTypeFieldListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemTypeFieldGetRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemTypeFieldShowArgs {
    #[facet(flatten)]
    pub args: AzureDevOpsWorkItemTypeFieldListArgs,
    /// Field reference name.
    #[facet(figue::positional)]
    pub field: AzureDevOpsWorkItemFieldName,
}

impl AzureDevOpsWorkItemTypeFieldShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self
            .args
            .authentication
            .resolve(&self.args.org, executor)
            .await?;
        let value = AzureDevOpsWorkItemTypeFieldGetRequest {
            org_url: Cow::Owned(self.args.org),
            project: self.args.project,
            auth_selection,
            work_item_type: self.args.work_item_type,
            field: self.field,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(value))
    }
}
