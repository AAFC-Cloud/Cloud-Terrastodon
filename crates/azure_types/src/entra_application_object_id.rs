use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct EntraApplicationObjectId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationObjectId);

cloud_terrastodon_registry::register_thing!(EntraApplicationObjectId);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationObjectId);
