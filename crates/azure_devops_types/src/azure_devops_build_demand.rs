use arbitrary::Arbitrary;

/// An agent capability demanded by a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#demand).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDemand {
    pub name: String,
    pub value: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDemand);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDemand);
