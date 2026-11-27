use super::entra_devops_permission_row::EntraDevOpsPermissionRow;
use cloud_terrastodon_azure_devops::AzureDevOpsIdentityPermissionReport;

/// An organization's matching identity and its derived table rows.
#[derive(Debug)]
pub(super) struct EntraDevOpsPermissionIdentity {
    pub display_name: String,
    pub report: AzureDevOpsIdentityPermissionReport,
    pub rows: Vec<EntraDevOpsPermissionRow>,
}

impl EntraDevOpsPermissionIdentity {
    pub fn new(display_name: String, report: AzureDevOpsIdentityPermissionReport) -> Self {
        let rows = report
            .entries
            .iter()
            .map(|entry| EntraDevOpsPermissionRow::new(entry, &report))
            .collect();
        Self {
            display_name,
            report,
            rows,
        }
    }
}
