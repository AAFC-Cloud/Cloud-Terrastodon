use crate::prelude::AzureDevOpsAgentPoolEntitlementId;
use crate::prelude::AzureDevOpsAgentPoolId;
use crate::prelude::AzureDevOpsAgentPoolName;
use crate::prelude::AzureDevOpsProjectId;

/// In Azure DevOps documentation, the grant for a given [`AzureDevOpsProject`] to use an [`AzureDevOpsAgentPool`] is called a "queue" but that's silly so we will call it an entitlement instead.
/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/distributedtask/queues/get?view=azure-devops-rest-7.1>
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolEntitlement {
    /// The queue id (this is NOT the same as the underlying pool id).
    pub id: AzureDevOpsAgentPoolEntitlementId,
    pub name: String,
    pub pool: AzureDevOpsAgentPoolEntitlementPoolReference,
    pub project_id: AzureDevOpsProjectId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolEntitlementPoolReference {
    pub id: AzureDevOpsAgentPoolId,
    pub is_hosted: bool,
    pub is_legacy: bool,
    pub name: AzureDevOpsAgentPoolName,
    pub options: String,
    pub pool_type: String,
    pub scope: String,
    pub size: usize,
}
