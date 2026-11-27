use cloud_terrastodon_azure_devops::AzureDevOpsIdentityPermissionEntry;
use cloud_terrastodon_azure_devops::AzureDevOpsIdentityPermissionReport;
use facet::Facet;

/// Searchable display columns for an Azure DevOps permission entry.
#[derive(Debug, Facet)]
pub(super) struct EntraDevOpsPermissionRow {
    pub namespace_id: String,
    pub namespace: String,
    pub token: String,
    pub assignment: &'static str,
    pub assignee: String,
    pub allow: String,
    pub deny: String,
}

impl EntraDevOpsPermissionRow {
    pub fn new(
        entry: &AzureDevOpsIdentityPermissionEntry,
        report: &AzureDevOpsIdentityPermissionReport,
    ) -> Self {
        Self {
            namespace_id: entry.namespace_id.to_string(),
            namespace: entry.namespace.clone(),
            token: entry.security_token.clone(),
            assignment: if entry.assignee == report.subject {
                "Direct"
            } else {
                "Parent group"
            },
            assignee: entry.assignee.to_string(),
            allow: entry.allow.join(", "),
            deny: entry.deny.join(", "),
        }
    }
}
