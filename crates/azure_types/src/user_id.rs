use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraUserId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraUserId);
