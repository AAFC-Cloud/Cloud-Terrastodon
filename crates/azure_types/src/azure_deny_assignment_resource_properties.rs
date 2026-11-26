use crate::AzureDenyAssignmentPermission;
use crate::AzureDenyAssignmentPrincipal;
use crate::EntraPrincipalId;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

/// Properties of an ARM [deny assignment](https://learn.microsoft.com/en-us/rest/api/authorization/deny-assignments/list-for-scope?view=rest-authorization-2022-04-01#denyassignment).
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDenyAssignmentResourceProperties {
    pub deny_assignment_name: String,
    pub description: String,
    pub do_not_apply_to_child_scopes: bool,
    pub exclude_principals: Vec<AzureDenyAssignmentPrincipal>,
    pub is_system_protected: bool,
    pub permissions: Vec<AzureDenyAssignmentPermission>,
    pub principals: Vec<AzureDenyAssignmentPrincipal>,
    pub scope: ScopeImpl,
    /// Conditions and audit metadata are omitted by the documented list example.
    pub condition: Option<String>,
    pub condition_version: Option<String>,
    pub created_by: Option<EntraPrincipalId>,
    pub created_on: Option<Timestamp>,
    pub updated_by: Option<EntraPrincipalId>,
    pub updated_on: Option<Timestamp>,
}

cloud_terrastodon_registry::register_thing!(AzureDenyAssignmentResourceProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureDenyAssignmentResourceProperties);
