use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraApplicationRegistrationId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationRegistrationId);

cloud_terrastodon_registry::register_thing!(EntraApplicationRegistrationId);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRegistrationId);
