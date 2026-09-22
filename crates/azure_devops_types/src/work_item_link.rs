use crate::AzureDevOpsWorkItemRelationTypeName;
use crate::WorkItemReference;
use arbitrary::Arbitrary;

/// A link between two work items.
#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct WorkItemLink {
    /// The type of link (optional).
    #[facet(skip_serializing_if = Option::is_none)]
    pub rel: Option<AzureDevOpsWorkItemRelationTypeName>,
    /// The source work item (optional).
    #[facet(skip_serializing_if = Option::is_none)]
    pub source: Option<WorkItemReference>,
    /// The target work item.
    pub target: WorkItemReference,
}
