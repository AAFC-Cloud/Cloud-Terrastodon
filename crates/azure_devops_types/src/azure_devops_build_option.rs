use crate::AzureDevOpsBuildOptionDefinitionReference;
use arbitrary::Arbitrary;
use std::collections::BTreeMap;

/// A behavior enabled or configured on a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildoption).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildOption {
    pub definition: AzureDevOpsBuildOptionDefinitionReference,
    pub enabled: Option<bool>,
    pub inputs: Option<BTreeMap<String, String>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildOption);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildOption);
