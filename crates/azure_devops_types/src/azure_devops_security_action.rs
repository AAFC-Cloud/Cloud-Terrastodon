use facet::Facet;
use uuid::Uuid;

/// A permission bit defined by [Security Namespaces Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/security-namespaces/query?view=azure-devops-rest-7.1#actiondefinition).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsSecurityAction {
    pub bit: i32,
    pub name: String,
    pub display_name: String,
    pub namespace_id: Uuid,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsSecurityAction);
