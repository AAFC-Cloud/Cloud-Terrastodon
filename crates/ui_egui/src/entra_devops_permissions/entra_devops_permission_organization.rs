use super::entra_devops_permission_identity::EntraDevOpsPermissionIdentity;
use crate::capture_gui_failure::capture_gui_failure;
use cloud_terrastodon_azure_devops::AzureDevOpsEntraPermissionOrganization;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use eyre::Report;
use teamy_tracing::DiagnosticReport;

/// Coverage of an Entra principal in one tracked Azure DevOps organization.
#[derive(Debug)]
pub(crate) struct EntraDevOpsPermissionOrganization {
    pub(super) organization: AzureDevOpsOrganizationUrl,
    pub(super) result:
        Result<Vec<Result<EntraDevOpsPermissionIdentity, DiagnosticReport>>, DiagnosticReport>,
}

impl EntraDevOpsPermissionOrganization {
    pub(super) fn from_scan(scan: AzureDevOpsEntraPermissionOrganization<Report>) -> Self {
        let result = scan
            .result
            .map(|identities| {
                identities
                    .into_iter()
                    .map(|identity| {
                        identity
                            .result
                            .map(|report| {
                                EntraDevOpsPermissionIdentity::new(identity.display_name, report)
                            })
                            .map_err(|error| {
                                capture_gui_failure(
                                    error,
                                    "Azure DevOps identity permissions lookup failed",
                                )
                            })
                    })
                    .collect()
            })
            .map_err(|error| {
                capture_gui_failure(error, "Azure DevOps organization permissions lookup failed")
            });
        Self {
            organization: scan.organization,
            result,
        }
    }
}
