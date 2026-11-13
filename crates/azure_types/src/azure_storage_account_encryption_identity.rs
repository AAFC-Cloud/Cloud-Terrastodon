use crate::EntraApplicationClientId;
use crate::ResourceId;
use arbitrary::Arbitrary;
use facet::Facet;

/// Identity selected to access a customer-managed encryption key.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#encryptionidentity).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountEncryptionIdentity {
    pub federated_identity_client_id: Option<EntraApplicationClientId>,
    pub user_assigned_identity: Option<ResourceId>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountEncryptionIdentity);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountEncryptionIdentity);
