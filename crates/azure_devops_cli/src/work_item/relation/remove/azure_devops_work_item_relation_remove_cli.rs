use crate::work_item::relation::show::AzureDevOpsWorkItemRelationShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationChange;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationChangeRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationSelector;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemRelationRemoveArgs {
    #[facet(flatten)]
    pub relation: AzureDevOpsWorkItemRelationShowArgs,
    /// Ask the server to validate without saving changes.
    #[facet(figue::named, default)]
    pub validate_only: bool,
    /// Do not fire any notifications for this change.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
    /// Require this revision before changing the work item.
    #[facet(figue::named)]
    pub if_rev: Option<i32>,
}

impl AzureDevOpsWorkItemRelationRemoveArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let target = self
            .relation
            .target
            .into_relation_url(&self.relation.org)
            .into_work_item()?;
        let auth_selection = self
            .relation
            .authentication
            .resolve(&self.relation.org, executor)
            .await?;
        let change =
            AzureDevOpsWorkItemRelationChange::Remove(AzureDevOpsWorkItemRelationSelector {
                rel: self.relation.relation_type,
                target,
            });
        let item = AzureDevOpsWorkItemRelationChangeRequest {
            org_url: Cow::Owned(self.relation.org),
            project: Some(self.relation.project),
            auth_selection,
            id: self.relation.id,
            change,
            validate_only: self.validate_only,
            suppress_notifications: self.suppress_notifications,
            if_rev: self.if_rev,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(item))
    }
}
