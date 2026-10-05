use crate::AzureDevOpsAgentPoolId;
use crate::AzureDevOpsAgentPoolName;
use arbitrary::Arbitrary;

/// The shallow agent pool embedded in a build queue.
///
/// See Microsoft's [TaskAgentPoolReference 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#taskagentpoolreference).
/// This build response shape does not require the ownership and configuration
/// metadata of a full [`crate::AzureDevOpsAgentPool`].
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolReference {
    pub id: AzureDevOpsAgentPoolId,
    pub name: AzureDevOpsAgentPoolName,
    pub is_hosted: bool,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolReference);
