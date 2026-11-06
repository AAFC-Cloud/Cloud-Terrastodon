use arbitrary::Arbitrary;
use facet::Facet;
/// Whether ARM eligible-child discovery includes all descendants of a management group.
#[derive(Eq, PartialEq, Debug, Default, Facet, Arbitrary)]
#[repr(u8)]
pub enum FetchChildrenBehaviour {
    #[default]
    Unspecified,
    GetAllChildren,
}
