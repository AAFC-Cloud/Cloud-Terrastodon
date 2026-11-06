use crate::WorkItemFieldReference;
use arbitrary::Arbitrary;
use facet::Facet;

/// A sort column.
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct WorkItemQuerySortColumn {
    /// The direction to sort by.
    pub descending: bool,
    /// A work item field.
    pub field: WorkItemFieldReference,
}
