use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct EntraServicePrincipalApplicationRoleId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraServicePrincipalApplicationRoleId);

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalApplicationRoleId);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalApplicationRoleId);
