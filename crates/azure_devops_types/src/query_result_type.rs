use arbitrary::Arbitrary;
use facet::Facet;

/// The result type of a query.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum QueryResultType {
    WorkItem,
    WorkItemLink,
}
