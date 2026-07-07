use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ConditionalAccessNamedLocationId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessNamedLocationId);

cloud_terrastodon_registry::register_thing!(ConditionalAccessNamedLocationId);
cloud_terrastodon_registry::register_arbitrary!(ConditionalAccessNamedLocationId);
