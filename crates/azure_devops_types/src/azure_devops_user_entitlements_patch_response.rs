use crate::AzureDevOpsLicenseEntitlementUpdateResponse;
use crate::AzureDevOpsUserEntitlementOperationResult;
use facet::Facet;

/// Response from the [7.1 user-entitlement update API](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#userentitlementspatchresponse).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsUserEntitlementsPatchResponse {
    pub is_success: bool,
    pub operation_results: Vec<AzureDevOpsUserEntitlementOperationResult>,
    pub user_entitlement: AzureDevOpsLicenseEntitlementUpdateResponse,
}
