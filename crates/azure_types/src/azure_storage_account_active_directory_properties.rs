use crate::AzureStorageAccountActiveDirectoryAccountType;
use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

/// Directory metadata used for Azure Files authentication.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#activedirectoryproperties).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountActiveDirectoryProperties {
    /// These fields depend on the selected directory mechanism; Entra-backed
    /// authentication can omit the on-premises AD details.
    pub account_type: Option<AzureStorageAccountActiveDirectoryAccountType>,
    pub azure_storage_sid: Option<String>,
    pub domain_guid: Option<Uuid>,
    pub domain_name: Option<String>,
    pub domain_sid: Option<String>,
    pub forest_name: Option<String>,
    pub net_bios_domain_name: Option<String>,
    pub sam_account_name: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountActiveDirectoryProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountActiveDirectoryProperties);
