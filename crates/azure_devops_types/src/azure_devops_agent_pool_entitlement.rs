use crate::AzureDevOpsAgentPoolEntitlementId;
use crate::AzureDevOpsAgentPoolId;
use crate::AzureDevOpsAgentPoolName;
use crate::AzureDevOpsProjectId;
use arbitrary::Arbitrary;

/// In Azure DevOps documentation, the grant for a given [`AzureDevOpsProject`] to use an [`AzureDevOpsAgentPool`] is called a "queue" but that's silly so we will call it an entitlement instead.
/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/distributedtask/queues/get?view=azure-devops-rest-7.1>
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPoolEntitlement {
    /// The queue id (this is NOT the same as the underlying pool id).
    pub id: AzureDevOpsAgentPoolEntitlementId,
    pub name: String,
    pub pool: AzureDevOpsAgentPoolEntitlementPoolReference,
    pub project_id: AzureDevOpsProjectId,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolEntitlement);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolEntitlement);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsAgentPoolEntitlement>);
