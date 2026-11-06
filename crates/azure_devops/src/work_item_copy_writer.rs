use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Performs copy writes and verification using the owning request's execution contract.
#[expect(
    clippy::double_must_use,
    reason = "async-trait adds #[must_use] to an already must-use future"
)]
#[async_trait]
pub(crate) trait WorkItemCopyWriter: Sync {
    async fn create(
        &self,
        work_item_type: AzureDevOpsWorkItemType,
        patch: AzureDevOpsJsonPatch,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<AzureDevOpsWorkItem>;
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()>;
    async fn read(
        &self,
        ids: Vec<AzureDevOpsWorkItemId>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<AzureDevOpsWorkItem>>;
}
