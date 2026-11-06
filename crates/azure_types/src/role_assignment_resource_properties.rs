use crate::EntraPrincipalId;
use crate::ResourceId;
use crate::RoleAssignmentPrincipalType;
use crate::RoleDefinitionId;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// [ARM role-assignment properties](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#roleassignment).
/// Conditions, description, delegation and audit metadata are absent from the documented minimal response.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct RoleAssignmentResourceProperties {
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: EntraPrincipalId,
    pub principal_type: RoleAssignmentPrincipalType,
    pub scope: ScopeImpl,
    pub condition: Option<String>,
    pub condition_version: Option<String>,
    pub description: Option<String>,
    pub delegated_managed_identity_resource_id: Option<ResourceId>,
    pub created_by: Option<EntraPrincipalId>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_by: Option<EntraPrincipalId>,
    pub updated_on: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentResourceProperties);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentResourceProperties);
