use crate::AzureDevOpsLicenseEntitlementUpdateResponse;
use crate::AzureDevOpsUserId;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// One result from the [7.1 user-entitlement patch operation](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#userentitlementoperationresult).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsUserEntitlementOperationResult {
    pub is_success: bool,
    /// The REST schema declares open error-code/message objects.
    pub errors: Vec<ArbitraryJson>,
    pub user_id: AzureDevOpsUserId,
    /// The documented example includes null for a successful secondary result.
    pub result: Option<AzureDevOpsLicenseEntitlementUpdateResponse>,
}
