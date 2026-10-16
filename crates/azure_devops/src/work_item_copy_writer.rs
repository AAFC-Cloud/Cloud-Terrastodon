use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use eyre::Result;

/// Performs copy writes and verification using the owning request's execution contract.
#[async_trait]
pub(crate) trait WorkItemCopyWriter: Sync {
    async fn create(
        &self,
        work_item_type: AzureDevOpsWorkItemType,
        patch: AzureDevOpsJsonPatch,
        cancellation: &CancellationScope,
    ) -> Result<AzureDevOpsWorkItem>;
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
        cancellation: &CancellationScope,
    ) -> Result<()>;
    async fn read(
        &self,
        ids: Vec<AzureDevOpsWorkItemId>,
        cancellation: &CancellationScope,
    ) -> Result<Vec<AzureDevOpsWorkItem>>;
}
