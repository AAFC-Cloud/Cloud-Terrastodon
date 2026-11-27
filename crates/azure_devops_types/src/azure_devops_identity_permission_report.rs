use crate::AzureDevOpsIdentityPermissionEntry;
use crate::AzureDevOpsLegacyIdentityDescriptor;
use facet::Facet;
use std::collections::BTreeMap;

/// ACL discovery results and coverage for an identity in one Azure DevOps organization.
/// [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1).
#[derive(Debug, Facet)]
pub struct AzureDevOpsIdentityPermissionReport {
    pub subject: AzureDevOpsLegacyIdentityDescriptor,
    pub parent_groups: Vec<AzureDevOpsLegacyIdentityDescriptor>,
    pub parent_memberships_available: bool,
    pub entries: Vec<AzureDevOpsIdentityPermissionEntry>,
    pub checked_namespace_count: usize,
    pub failed_namespaces: BTreeMap<String, String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityPermissionReport);
