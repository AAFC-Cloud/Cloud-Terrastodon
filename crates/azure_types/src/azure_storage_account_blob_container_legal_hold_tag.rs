use crate::AzureTenantId;
use crate::EntraUserId;
use crate::UserPrincipalName;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// An individual legal-hold tag and the identity that added it.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#tagproperty).
/// A tag identifies its value, actor, and timestamp. The [GetContainers examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#examples)
/// omit UPN while retaining the actor's object and tenant IDs.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerLegalHoldTag {
    pub tag: String,
    pub timestamp: DateTime<Utc>,
    pub object_identifier: EntraUserId,
    pub tenant_id: AzureTenantId,
    /// Not included in the official tag examples.
    pub upn: Option<UserPrincipalName>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerLegalHoldTag);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerLegalHoldTag);
