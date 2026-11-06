use crate::AzureDevOpsWorkItemId;
use arbitrary::Arbitrary;
use facet::Facet;

/// Contains reference to a work item.
#[derive(Debug, Arbitrary, Facet)]
pub struct WorkItemReference {
    /// Work item ID.
    pub id: AzureDevOpsWorkItemId,
    /// REST API URL of the resource.
    pub url: String,
}
