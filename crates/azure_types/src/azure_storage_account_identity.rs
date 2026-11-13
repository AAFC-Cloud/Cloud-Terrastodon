use crate::AzureStorageAccountIdentityType;
use crate::AzureStorageAccountUserAssignedIdentity;
use crate::AzureTenantId;
use crate::EntraServicePrincipalObjectId;
use crate::ResourceId;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;

/// Managed identities associated with a storage account.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#identity).
/// Only the identity type is required; IDs and user-assigned identities are
/// conditional on the selected identity configuration.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountIdentity {
    #[facet(rename = "type")]
    pub identity_type: AzureStorageAccountIdentityType,
    pub principal_id: Option<EntraServicePrincipalObjectId>,
    pub tenant_id: Option<AzureTenantId>,
    pub user_assigned_identities:
        Option<HashMap<ResourceId, AzureStorageAccountUserAssignedIdentity>>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountIdentity);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountIdentity);
