use crate::AzureStorageAccountExtendedLocationType;
use arbitrary::Arbitrary;
use facet::Facet;

/// Extended deployment location reported for a storage account.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#extendedlocation).
/// Both fields are optional; an omitted name is not the account's primary region.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountExtendedLocation {
    /// Service-defined extended-location name, distinct from the Azure region name.
    pub name: Option<String>,
    #[facet(rename = "type")]
    pub location_type: Option<AzureStorageAccountExtendedLocationType>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountExtendedLocation);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountExtendedLocation);
