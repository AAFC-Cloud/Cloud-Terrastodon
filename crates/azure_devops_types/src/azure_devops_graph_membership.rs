use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsReferenceLinks;
use facet::Facet;

/// A direct Graph membership edge returned by the [7.1 membership API](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/memberships/list?view=azure-devops-rest-7.1#graphmembership).
#[derive(Debug, Clone, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsGraphMembership {
    pub container_descriptor: AzureDevOpsDescriptor,
    pub member_descriptor: AzureDevOpsDescriptor,
    #[facet(rename = "_links")]
    /// Omitted in the documented 7.1 list responses.
    pub links: Option<AzureDevOpsReferenceLinks>,
}
