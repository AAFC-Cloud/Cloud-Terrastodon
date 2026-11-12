use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;

/// The tag dictionary on an ARM tags resource.
/// [Tags schema](https://learn.microsoft.com/rest/api/resources/tags/get-at-scope?view=rest-resources-2021-04-01#tags).
#[derive(Debug, Facet, Arbitrary)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceTagsProperties {
    pub tags: HashMap<String, String>,
}
cloud_terrastodon_registry::register_thing!(AzureResourceTagsProperties);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTagsProperties);
