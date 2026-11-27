use crate::EntraServicePrincipalApplicationRoleId;
use arbitrary::Arbitrary;
use facet::Facet;

/// A role declared on an Entra [application or service principal](https://learn.microsoft.com/graph/api/resources/approle?view=graph-rest-1.0).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraServicePrincipalApplicationRole {
    pub allowed_member_types: Vec<String>,
    pub description: String,
    pub display_name: String,
    pub id: EntraServicePrincipalApplicationRoleId,
    pub is_enabled: bool,
    pub origin: String,
    /// [Roles without a token claim value](https://learn.microsoft.com/entra/identity-platform/enterprise-app-role-management) can contain null.
    pub value: Option<String>,
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalApplicationRole);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalApplicationRole);
