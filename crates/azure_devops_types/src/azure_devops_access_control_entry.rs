use crate::AzureDevOpsAccessControlEntryExtendedInformation;
use crate::AzureDevOpsLegacyIdentityDescriptor;
use facet::Facet;

/// Explicit permission bits from [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1#accesscontrolentry).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAccessControlEntry {
    pub descriptor: AzureDevOpsLegacyIdentityDescriptor,
    pub allow: i32,
    pub deny: i32,
    #[facet(rename = "extendedInfo")]
    pub extended_info: Option<AzureDevOpsAccessControlEntryExtendedInformation>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccessControlEntry);
