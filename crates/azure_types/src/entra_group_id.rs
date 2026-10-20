use arbitrary::Arbitrary;

/// The object ID of an Entra [group](https://learn.microsoft.com/en-us/graph/api/resources/group?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraGroupId(pub uuid::Uuid);

crate::impl_uuid_newtype!(EntraGroupId);

cloud_terrastodon_registry::register_thing!(EntraGroupId);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupId);
