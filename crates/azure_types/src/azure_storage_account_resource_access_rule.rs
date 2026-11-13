use crate::AzureTenantId;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;

/// A resource instance admitted by storage network rules.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#resourceaccessrule).
/// Both identities are optional in the [official SDK's schema-derived model](https://github.com/Azure/azure-sdk-for-go/blob/main/sdk/resourcemanager/storage/armstorage/models.go#L3773).
/// An omitted identity must not be replaced with the caller's tenant or another resource.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountResourceAccessRule {
    pub resource_id: Option<ScopeImpl>,
    pub tenant_id: Option<AzureTenantId>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountResourceAccessRule);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountResourceAccessRule);
