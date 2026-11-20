use crate::RolePermissionActionSet;
use arbitrary::Arbitrary;
use facet::Facet;

/// An ARM [deny assignment permission](https://learn.microsoft.com/en-us/rest/api/authorization/deny-assignments/list-for-scope?view=rest-authorization-2022-04-01#denyassignmentpermission).
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDenyAssignmentPermission {
    pub actions: RolePermissionActionSet,
    pub not_actions: RolePermissionActionSet,
    pub data_actions: RolePermissionActionSet,
    pub not_data_actions: RolePermissionActionSet,
    /// Conditions are omitted by the documented list example.
    pub condition: Option<String>,
    pub condition_version: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDenyAssignmentPermission);
cloud_terrastodon_registry::register_arbitrary!(AzureDenyAssignmentPermission);
