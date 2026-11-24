use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsReferenceLinks;
use arbitrary::Arbitrary;
use facet::Facet;

/// A graph descriptor resolved from a storage key.
///
/// See [GraphDescriptorResult (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/descriptors/get?view=azure-devops-rest-7.1#graphdescriptorresult).
#[derive(Debug, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsGraphDescriptorResult {
    #[facet(rename = "_links")]
    pub links: AzureDevOpsReferenceLinks,
    pub value: AzureDevOpsDescriptor,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGraphDescriptorResult);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGraphDescriptorResult);
