use arbitrary::Arbitrary;

/// The result type of a query.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum QueryResultType {
    WorkItem,
    WorkItemLink,
}
