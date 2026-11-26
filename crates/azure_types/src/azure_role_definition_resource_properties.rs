use crate::EntraPrincipalId;
use crate::RoleDefinitionKind;
use crate::RolePermissions;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

/// Properties of an ARM [role definition](https://learn.microsoft.com/en-us/rest/api/authorization/role-definitions/list?view=rest-authorization-2022-04-01#roledefinition).
#[derive(Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureRoleDefinitionResourceProperties {
    pub role_name: String,
    #[facet(rename = "type")]
    pub kind: RoleDefinitionKind,
    pub description: String,
    pub assignable_scopes: Vec<ScopeImpl>,
    pub permissions: Vec<RolePermissions>,
    /// Creation and update metadata are omitted by the documented list example.
    pub created_by: Option<EntraPrincipalId>,
    pub created_on: Option<Timestamp>,
    pub updated_by: Option<EntraPrincipalId>,
    pub updated_on: Option<Timestamp>,
}

cloud_terrastodon_registry::register_thing!(AzureRoleDefinitionResourceProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleDefinitionResourceProperties);
