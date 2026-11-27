use crate::AzureDevOpsLegacyIdentityDescriptor;
use facet::Facet;
use uuid::Uuid;

/// Explicit ACL entries affecting an identity or one of its Azure DevOps parent groups.
/// [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1).
#[derive(Debug, Facet)]
pub struct AzureDevOpsIdentityPermissionEntry {
    pub namespace_id: Uuid,
    pub namespace: String,
    pub security_token: String,
    pub assignee: AzureDevOpsLegacyIdentityDescriptor,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityPermissionEntry);
