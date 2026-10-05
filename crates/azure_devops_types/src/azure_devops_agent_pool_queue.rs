use crate::AzureDevOpsAgentPoolQueueId;
use crate::AzureDevOpsAgentPoolQueueName;
use crate::AzureDevOpsAgentPoolQueueUrl;
use crate::AzureDevOpsAgentPoolReference;
use crate::AzureDevOpsReferenceLinks;
use arbitrary::Arbitrary;

/// A build definition's default agent queue and its shallow pool reference.
///
/// See Microsoft's [AgentPoolQueue 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#agentpoolqueue).
/// Hypermedia metadata can be omitted from embedded references, as represented
/// by the [official SDK interface](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAgentPoolQueue {
    pub id: AzureDevOpsAgentPoolQueueId,
    pub name: AzureDevOpsAgentPoolQueueName,
    pub pool: AzureDevOpsAgentPoolReference,
    pub url: Option<AzureDevOpsAgentPoolQueueUrl>,
    #[facet(rename = "_links")]
    pub links: Option<AzureDevOpsReferenceLinks>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolQueue);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolQueue);
