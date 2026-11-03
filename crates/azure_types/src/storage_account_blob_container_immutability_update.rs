use crate::AzureTenantId;
use crate::EntraUserId;
use crate::StorageAccountBlobContainerImmutabilityUpdateType;
use crate::UserPrincipalName;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// An audited change to a container's immutability policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#updatehistoryproperty).
/// A history entry identifies the change, retention period, actor, and time.
/// The [GetContainers examples](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/get?view=rest-storagerp-2025-06-01#examples)
/// omit UPN and omit append settings in the basic example.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct StorageAccountBlobContainerImmutabilityUpdate {
    pub update: StorageAccountBlobContainerImmutabilityUpdateType,
    pub immutability_period_since_creation_in_days: i32,
    pub timestamp: DateTime<Utc>,
    pub object_identifier: EntraUserId,
    pub tenant_id: AzureTenantId,
    /// Not included in the official history examples, even when the actor ID is present.
    pub upn: Option<UserPrincipalName>,
    /// Absent in the basic GetContainers history entries.
    pub allow_protected_append_writes: Option<bool>,
    /// Included by the protected-append example, omitted by basic GetContainers.
    pub allow_protected_append_writes_all: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerImmutabilityUpdate);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerImmutabilityUpdate);
