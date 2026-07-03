#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraGroupId(pub uuid::Uuid);

crate::impl_uuid_newtype!(EntraGroupId);

cloud_terrastodon_registry::register_thing!(EntraGroupId);
