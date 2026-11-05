use crate::ResourceType;

/// Name and resource kind submitted to ARM's check-name-availability operation.
///
/// [Storage request](https://learn.microsoft.com/rest/api/storagerp/storage-accounts/check-name-availability?view=rest-storagerp-2025-06-01#storageaccountchecknameavailabilityparameters)
/// and [vault request](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01#vaultchecknameavailabilityparameters).
#[derive(facet::Facet)]
pub struct AzureResourceNameAvailabilityBody<Name> {
    pub name: Name,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
}
