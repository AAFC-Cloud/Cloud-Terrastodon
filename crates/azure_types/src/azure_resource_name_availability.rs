use crate::AzureResourceNameUnavailableReason;
use arbitrary::Arbitrary;

/// The shared ARM name-availability response for storage accounts and vaults.
///
/// [Storage 2025-06-01](https://learn.microsoft.com/rest/api/storagerp/storage-accounts/check-name-availability?view=rest-storagerp-2025-06-01#checknameavailabilityresult)
/// and [Key Vault 2024-11-01](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01#checknameavailabilityresult)
/// omit the reason and explanatory message when the name is available.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceNameAvailability {
    pub name_available: bool,
    pub reason: Option<AzureResourceNameUnavailableReason>,
    pub message: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceNameAvailability);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceNameAvailability);
