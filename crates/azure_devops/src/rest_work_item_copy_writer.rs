use crate::azure_devops_work_item_create_request::AzureDevOpsWorkItemCreateRequest;
use crate::azure_devops_work_item_relation_change::AzureDevOpsWorkItemRelationChange;
use crate::azure_devops_work_item_relation_change_request::AzureDevOpsWorkItemRelationChangeRequest;
use crate::azure_devops_work_items_get_request::AzureDevOpsWorkItemsGetRequest;
use crate::work_item_copy_writer::WorkItemCopyWriter;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;

/// Executes copy operations through the SDK using each phase's supplied scope.
#[derive(Debug)]
pub(crate) struct RestWorkItemCopyWriter {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub auth_context: AzureDevOpsAuthContext,
    pub suppress_notifications: bool,
}

#[async_trait]
impl WorkItemCopyWriter for RestWorkItemCopyWriter {
    async fn create(
        &self,
        work_item_type: AzureDevOpsWorkItemType,
        patch: AzureDevOpsJsonPatch,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureDevOpsWorkItem> {
        let response = AzureDevOpsWorkItemCreateRequest {
            org_url: Cow::Borrowed(&self.org_url),
            project: self.project.clone(),
            auth_context: Cow::Borrowed(&self.auth_context),
            work_item_type,
            patch,
            validate_only: false,
            suppress_notifications: self.suppress_notifications,
        }
        .execute(executor)
        .await?;
        facet_json::from_str(response.as_ref())
            .map_err(|error| eyre::eyre!("Created item response could not be decoded: {error}"))
    }
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        AzureDevOpsWorkItemRelationChangeRequest {
            org_url: Cow::Borrowed(&self.org_url),
            project: Some(self.project.clone()),
            auth_context: Cow::Borrowed(&self.auth_context),
            id: source,
            change: AzureDevOpsWorkItemRelationChange::Create(relation),
            validate_only: false,
            suppress_notifications: self.suppress_notifications,
            if_rev: None,
        }
        .execute(executor)
        .await?;
        Ok(())
    }
    async fn read(
        &self,
        ids: Vec<AzureDevOpsWorkItemId>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<AzureDevOpsWorkItem>> {
        AzureDevOpsWorkItemsGetRequest {
            org_url: Cow::Borrowed(&self.org_url),
            project: Some(self.project.clone()),
            auth_context: Cow::Borrowed(&self.auth_context),
            ids,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .execute(executor)
        .await
    }
}
