use crate::azure_resource_created_by_type::AzureResourceCreatedByType;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// ARM provenance metadata for a resource's creation and latest modification.
///
/// Shared by [Storage RP 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#systemdata)
/// and [Policy Insights 2024-10-01](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource?view=rest-policyinsights-2024-10-01#systemdata).
/// The [ARM common schema](https://github.com/Azure/azure-rest-api-specs/blob/main/specification/common-types/resource-management/v5/types.json)
/// describes the six fields of a present provenance record. They are required
/// here; a containing response models absence of the entire record separately.
/// Identity strings have no UUID constraint.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceSystemData {
    /// Service-provided identity of the resource's creator.
    pub created_by: String,
    pub created_by_type: AzureResourceCreatedByType,
    /// Creation time, represented in UTC.
    pub created_at: DateTime<Utc>,
    /// Service-provided identity responsible for the latest modification.
    pub last_modified_by: String,
    pub last_modified_by_type: AzureResourceCreatedByType,
    /// Latest modification time, represented in UTC.
    pub last_modified_at: DateTime<Utc>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceSystemData);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceSystemData);
