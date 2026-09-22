use crate::AzureDevOpsWorkItemFieldName;
use arbitrary::Arbitrary;

/// Reference to a field in a work item.
#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct WorkItemFieldReference {
    /// The reference name of the field.
    pub reference_name: AzureDevOpsWorkItemFieldName,
    /// The friendly name of the field.
    pub name: String,
    /// The REST URL of the resource.
    pub url: String,
}
