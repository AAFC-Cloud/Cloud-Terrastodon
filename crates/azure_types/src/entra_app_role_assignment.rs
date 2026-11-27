use crate::EntraAppRoleAssignmentId;
use crate::EntraAppRoleAssignmentPrincipalType;
use crate::EntraPrincipalId;
use crate::EntraServicePrincipalApplicationRoleId;
use crate::EntraServicePrincipalObjectId;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;
use url::Url;

/// An assignment of an [app role to a principal](https://learn.microsoft.com/graph/api/resources/approleassignment?view=graph-rest-1.0#properties).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraAppRoleAssignment {
    pub id: EntraAppRoleAssignmentId,
    pub app_role_id: EntraServicePrincipalApplicationRoleId,
    pub created_date_time: Timestamp,
    pub deleted_date_time: Option<Timestamp>,
    pub principal_id: EntraPrincipalId,
    pub principal_display_name: String,
    pub principal_type: EntraAppRoleAssignmentPrincipalType,
    pub resource_id: EntraServicePrincipalObjectId,
    pub resource_display_name: String,
    #[facet(rename = "@odata.id")]
    #[arbitrary(default)]
    pub odata_id: Option<Url>,
    #[facet(rename = "@odata.type")]
    pub odata_type: Option<String>,
}

cloud_terrastodon_registry::register_thing!(EntraAppRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(EntraAppRoleAssignment);
