use crate::AzureDevOpsWorkItemId;
use arbitrary::Arbitrary;

/// Contains reference to a work item.
#[derive(Debug, Arbitrary, facet::Facet)]
pub struct WorkItemReference {
    /// Work item ID.
    pub id: AzureDevOpsWorkItemId,
    /// REST API URL of the resource.
    pub url: String,
}
