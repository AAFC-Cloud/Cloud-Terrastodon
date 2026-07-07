#[derive(Debug, arbitrary::Arbitrary, facet::Facet)]
pub struct MicrosoftGraphEntity<Id> {
    pub id: Id,
}
