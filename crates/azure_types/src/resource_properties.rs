use crate::ArbitraryJson;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;

/// Properties shared by the mixed-resource inventory from [Azure Resource Graph].
///
/// [Azure Resource Graph]: https://learn.microsoft.com/en-us/azure/governance/resource-graph/concepts/query-language
#[derive(Debug, Clone, Default, Eq, PartialEq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ResourceProperties {
    /// Friendly name when the resource type supplies one, as for [management groups].
    ///
    /// [management groups]: https://learn.microsoft.com/en-us/rest/api/managementgroups/management-groups/get?view=rest-managementgroups-2020-05-01#managementgroup
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub display_name: Option<String>,
    /// Provisioning state reported by the resource provider, using its own vocabulary.
    /// See [storage account properties] for one example.
    ///
    /// [storage account properties]: https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#storageaccount
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub provisioning_state: Option<String>,
    /// Remaining properties belong to individual resource providers.
    #[facet(flatten)]
    #[arbitrary(default)]
    pub other: HashMap<String, ArbitraryJson>,
}

impl From<Option<ResourceProperties>> for ResourceProperties {
    fn from(value: Option<ResourceProperties>) -> Self {
        value.unwrap_or_default()
    }
}

impl From<&ResourceProperties> for Option<ResourceProperties> {
    fn from(value: &ResourceProperties) -> Self {
        Some(value.clone())
    }
}

cloud_terrastodon_registry::register_thing!(ResourceProperties);
cloud_terrastodon_registry::register_arbitrary!(ResourceProperties);
