use crate::azure_devops_work_item_create_request::AzureDevOpsWorkItemCreateRequest;
use crate::azure_devops_work_item_relation_change::AzureDevOpsWorkItemRelationChange;
use crate::azure_devops_work_item_relation_change_request::AzureDevOpsWorkItemRelationChangeRequest;
use crate::azure_devops_work_items_get_request::AzureDevOpsWorkItemsGetRequest;
use crate::work_item_copy_writer::WorkItemCopyWriter;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
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
        cancellation: &CancellationScope,
    ) -> Result<AzureDevOpsWorkItem> {
        let response = AzureDevOpsWorkItemCreateRequest {
            cancellation: cancellation.clone(),
            org_url: Cow::Borrowed(&self.org_url),
            project: self.project.clone(),
            auth_context: Cow::Borrowed(&self.auth_context),
            work_item_type,
            patch,
            validate_only: false,
            suppress_notifications: self.suppress_notifications,
        }
        .await?;
        facet_json::from_str(response.as_ref())
            .map_err(|error| eyre::eyre!("Created item response could not be decoded: {error}"))
    }
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        AzureDevOpsWorkItemRelationChangeRequest {
            cancellation: cancellation.clone(),
            org_url: Cow::Borrowed(&self.org_url),
            project: Some(self.project.clone()),
            auth_context: Cow::Borrowed(&self.auth_context),
            id: source,
            change: AzureDevOpsWorkItemRelationChange::Create(relation),
            validate_only: false,
            suppress_notifications: self.suppress_notifications,
            if_rev: None,
        }
        .await?;
        Ok(())
    }
    async fn read(
        &self,
        ids: Vec<AzureDevOpsWorkItemId>,
        cancellation: &CancellationScope,
    ) -> Result<Vec<AzureDevOpsWorkItem>> {
        AzureDevOpsWorkItemsGetRequest {
            cancellation: cancellation.clone(),
            org_url: Cow::Borrowed(&self.org_url),
            project: Some(self.project.clone()),
            auth_context: Cow::Borrowed(&self.auth_context),
            ids,
            expand: Default::default(),
            fields: Vec::new(),
            as_of: None,
        }
        .await
    }
}
