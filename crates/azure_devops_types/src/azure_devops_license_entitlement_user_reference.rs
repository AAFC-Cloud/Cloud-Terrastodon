use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsIdentityDirectoryAlias;
use crate::AzureDevOpsIdentityDisplayName;
use crate::AzureDevOpsIdentityUrl;
use crate::AzureDevOpsReferenceLinks;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// The [GraphUser](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/search-user-entitlements?view=azure-devops-rest-7.1#graphuser) referenced by an Azure DevOps 7.1 user entitlement.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsLicenseEntitlementUserReference {
    #[facet(rename = "_links")]
    pub links: Option<AzureDevOpsReferenceLinks>,
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: Option<AzureDevOpsIdentityDirectoryAlias>,
    pub display_name: AzureDevOpsIdentityDisplayName,
    pub domain: Option<String>,
    pub is_deleted_in_origin: Option<bool>,
    pub legacy_descriptor: Option<String>,
    pub mail_address: Option<String>,
    pub meta_type: Option<String>,
    /// Also exposed by the [Azure DevOps v7.1 Python SDK](https://github.com/microsoft/azure-devops-python-api/blob/dev/azure-devops/azure/devops/v7_1/member_entitlement_management/models.py#L1494-L1514).
    pub metadata_update_date: Option<DateTime<Utc>>,
    pub origin: Option<String>,
    pub origin_id: Option<String>,
    pub principal_name: String,
    pub subject_kind: Option<String>,
    pub url: Option<AzureDevOpsIdentityUrl>,
}
