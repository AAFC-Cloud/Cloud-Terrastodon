use crate::AzureDevOpsAgentPoolQueueId;
use arbitrary::Arbitrary;

/// An agent-queue resource reference used by a build process.
///
/// REST API context (7.1): the containing [BuildDefinition.process property](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition).
/// That REST page documents the base process but omits this shallow YAML
/// resource-reference shape.
/// Supplemental JavaScript shape: [AgentPoolQueueReference](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/agentpoolqueuereference).
/// TypeScript source: [AgentPoolQueueReference](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L45).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolQueueReference {
    pub id: Option<AzureDevOpsAgentPoolQueueId>,
    pub alias: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolQueueReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolQueueReference);
