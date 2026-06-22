#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ConditionalAccessNamedLocationId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessNamedLocationId);
