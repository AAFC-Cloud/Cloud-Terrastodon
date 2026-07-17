use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct EntraApplicationObjectId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationObjectId);

cloud_terrastodon_registry::register_thing!(EntraApplicationObjectId);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationObjectId);
