use arbitrary::Arbitrary;
use uuid::Uuid;

/// The object ID of an Entra [organizational contact](https://learn.microsoft.com/en-us/graph/api/resources/orgcontact?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraOrganizationalContactId(Uuid);

crate::impl_uuid_newtype!(EntraOrganizationalContactId);

cloud_terrastodon_registry::register_thing!(EntraOrganizationalContactId);
cloud_terrastodon_registry::register_arbitrary!(EntraOrganizationalContactId);
