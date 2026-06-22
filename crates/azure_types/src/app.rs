use arbitrary::Arbitrary;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AppId(uuid::Uuid);

crate::impl_uuid_newtype!(AppId);
