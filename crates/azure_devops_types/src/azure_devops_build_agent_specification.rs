use arbitrary::Arbitrary;

/// An agent specification used by a designer build process.
///
/// REST API schema (7.1): [AgentSpecification](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#agentspecification).
/// TypeScript source: [AgentSpecification](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L78).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildAgentSpecification {
    pub identifier: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildAgentSpecification);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildAgentSpecification);
