use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use facet::Facet;

/// User count and estimated monthly cost for one license kind.
#[derive(Debug, Clone, PartialEq, Facet)]
pub struct AzureDevOpsLicenseSummaryRow {
    pub license: AzureDevOpsLicenseType,
    pub count: usize,
    pub cost_per_user_cad: f64,
    pub total_cost_cad: f64,
}
