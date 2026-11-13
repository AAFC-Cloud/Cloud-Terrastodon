use crate::EntraApplicationClientId;
use crate::EntraServicePrincipalObjectId;
use arbitrary::Arbitrary;
use facet::Facet;

/// Entra identity IDs reported for an attached user-assigned managed identity.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#userassignedidentity).
/// The enclosing map identifies the ARM resource; these read-only IDs may be omitted.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountUserAssignedIdentity {
    pub client_id: Option<EntraApplicationClientId>,
    pub principal_id: Option<EntraServicePrincipalObjectId>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountUserAssignedIdentity);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountUserAssignedIdentity);
