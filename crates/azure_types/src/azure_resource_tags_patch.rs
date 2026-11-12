use crate::AzureResourceTagsPatchOperation;
use crate::AzureResourceTagsProperties;
use facet::Facet;

/// Body of the ARM tags PATCH operation.
/// [TagsPatchResource](https://learn.microsoft.com/rest/api/resources/tags/update-at-scope?view=rest-resources-2021-04-01#tagspatchresource).
#[derive(Debug, Facet)]
pub struct AzureResourceTagsPatch {
    pub operation: AzureResourceTagsPatchOperation,
    pub properties: AzureResourceTagsProperties,
}
cloud_terrastodon_registry::register_thing!(AzureResourceTagsPatch);
