use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct EntraApplicationClientId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationClientId);

cloud_terrastodon_registry::register_thing!(EntraApplicationClientId);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationClientId);
