use arbitrary::Arbitrary;

/// The type of query.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum QueryType {
    Flat,
    OneHop,
    Tree,
}
