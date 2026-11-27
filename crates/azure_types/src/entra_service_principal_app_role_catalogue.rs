use crate::EntraApplicationClientId;
use crate::EntraServicePrincipalApplicationRole;
use crate::EntraServicePrincipalObjectId;
use arbitrary::Arbitrary;
use facet::Facet;
use url::Url;

/// The app roles exposed by a [service principal](https://learn.microsoft.com/graph/api/resources/serviceprincipal?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraServicePrincipalAppRoleCatalogue {
    #[facet(rename = "@odata.context")]
    #[arbitrary(default)]
    pub context: Option<Url>,
    pub id: EntraServicePrincipalObjectId,
    pub app_id: EntraApplicationClientId,
    pub display_name: String,
    pub app_roles: Vec<EntraServicePrincipalApplicationRole>,
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalAppRoleCatalogue);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalAppRoleCatalogue);
