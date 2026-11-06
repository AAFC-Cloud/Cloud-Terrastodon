use arbitrary::Arbitrary;
use facet::Facet;

/// The type of query.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum QueryType {
    Flat,
    OneHop,
    Tree,
}
