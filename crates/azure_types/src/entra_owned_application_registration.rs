use crate::EntraApplicationClientId;
use crate::EntraApplicationObjectId;
use arbitrary::Arbitrary;
use facet::Facet;

/// A Graph [application](https://learn.microsoft.com/graph/api/resources/application?view=graph-rest-1.0#properties)
/// projected from a user's [owned objects](https://learn.microsoft.com/graph/api/user-list-ownedobjects?view=graph-rest-1.0).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraOwnedApplicationRegistration {
    pub id: EntraApplicationObjectId,
    /// Graph can return only the ID and type when the caller has limited access.
    pub app_id: Option<EntraApplicationClientId>,
    pub display_name: Option<String>,
    #[facet(rename = "@odata.type")]
    pub odata_type: Option<String>,
}

cloud_terrastodon_registry::register_thing!(EntraOwnedApplicationRegistration);
cloud_terrastodon_registry::register_arbitrary!(EntraOwnedApplicationRegistration);
