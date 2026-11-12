use crate::work_item::field::show::AzureDevOpsWorkItemFieldShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemUpdateRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemFieldRemoveArgs {
    #[facet(flatten)]
    pub field: AzureDevOpsWorkItemFieldShowArgs,
    /// Ask the server to validate without saving changes.
    #[facet(figue::named, default)]
    pub validate_only: bool,
    /// Do not fire any notifications for this change.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
    /// Require this revision before updating the work item.
    #[facet(figue::named)]
    pub if_rev: Option<i32>,
}

impl AzureDevOpsWorkItemFieldRemoveArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let patch = AzureDevOpsJsonPatch::try_from(vec![AzureDevOpsJsonPatchOperation::Remove {
            path: format!(
                "/fields/{}",
                self.field.field.replace('~', "~0").replace('/', "~1")
            ),
        }])?;
        let auth_selection = self
            .field
            .authentication
            .resolve(&self.field.org, executor)
            .await?;
        let item = AzureDevOpsWorkItemUpdateRequest {
            org_url: Cow::Owned(self.field.org),
            project: Some(self.field.project),
            auth_selection,
            id: self.field.id,
            patch,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
            if_rev: self.if_rev,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(item))
    }
}
