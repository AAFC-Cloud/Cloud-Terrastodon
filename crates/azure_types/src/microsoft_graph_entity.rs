use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Debug, Arbitrary, Facet)]
pub struct MicrosoftGraphEntity<Id> {
    pub id: Id,
}
