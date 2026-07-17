use arbitrary::Arbitrary;
use facet::Facet;

use crate::EntraServicePrincipalApplicationRoleId;

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraServicePrincipalApplicationRole {
    pub allowed_member_types: Vec<String>,
    pub description: String,
    pub display_name: String,
    pub id: EntraServicePrincipalApplicationRoleId,
    pub is_enabled: bool,
    pub origin: String,
    pub value: String,
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalApplicationRole);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalApplicationRole);
