use crate::WorkItemFieldReference;
use arbitrary::Arbitrary;

/// A sort column.
#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct WorkItemQuerySortColumn {
    /// The direction to sort by.
    pub descending: bool,
    /// A work item field.
    pub field: WorkItemFieldReference,
}
