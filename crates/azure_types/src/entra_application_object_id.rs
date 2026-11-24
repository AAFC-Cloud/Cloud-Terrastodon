use arbitrary::Arbitrary;
use facet::Facet;

/// Directory object ID of an application registration.
/// [Microsoft Graph application `id`](https://learn.microsoft.com/en-us/graph/api/resources/application?view=graph-rest-1.0#properties)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct EntraApplicationObjectId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationObjectId);

cloud_terrastodon_registry::register_thing!(EntraApplicationObjectId);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationObjectId);
