use crate::AzureStorageAccountBlobContainerMigrationState;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use facet::Facet;

/// Object-level immutability configuration and migration status.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#immutablestoragewithversioning).
/// The [creation response](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/create?view=rest-storagerp-2025-06-01#putcontainerwithobjectlevelworm)
/// includes enabled but omits migration status and the enablement timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerImmutableStorageWithVersioning {
    pub enabled: bool,
    /// Not returned in the documented object-level WORM creation response.
    pub time_stamp: Option<DateTime<Utc>>,
    /// Relevant to migration; omitted when object-level WORM is enabled at creation.
    pub migration_state: Option<AzureStorageAccountBlobContainerMigrationState>,
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountBlobContainerImmutableStorageWithVersioning
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountBlobContainerImmutableStorageWithVersioning
);
