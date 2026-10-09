use crate::AzureDevOpsBuildOptionId;
use arbitrary::Arbitrary;

/// The UUID reference to an optional build behavior.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildoptiondefinitionreference).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildOptionDefinitionReference {
    pub id: AzureDevOpsBuildOptionId,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildOptionDefinitionReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildOptionDefinitionReference);
