use arbitrary::Arbitrary;

use crate::{
    EntraServicePrincipalApplicationResourceSpecificPermission,
    EntraServicePrincipalApplicationRole,
};

/// Application permissions exposed by an Entra service principal.
///
/// The entries are intentionally retained as raw Facet JSON for now because
/// Microsoft Graph uses different shapes for app roles and resource-specific
/// application permissions.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraServicePrincipalApplicationPermissions {
    pub app_roles: Vec<EntraServicePrincipalApplicationRole>,
    pub resource_specific_application_permissions:
        Vec<EntraServicePrincipalApplicationResourceSpecificPermission>,
}

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalApplicationPermissions);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalApplicationPermissions);
