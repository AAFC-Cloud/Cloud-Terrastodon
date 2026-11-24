use crate::AzureDevOpsUserLicenseEntitlement;
use facet::Facet;
use std::sync::Arc;

/// A [page of Azure DevOps 7.1 user entitlements](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/search-user-entitlements?view=azure-devops-rest-7.1#pageduserentitlementslist).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsUserLicenseEntitlementPage {
    pub items: Vec<Arc<AzureDevOpsUserLicenseEntitlement>>,
    pub continuation_token: Option<String>,
    /// Total inventory count reported by the service, across pages.
    pub total_count: i32,
}
