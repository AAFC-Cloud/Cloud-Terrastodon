use crate::AzureDevOpsAgentPoolQueueReference;
use crate::AzureDevOpsSecureFileReference;
use crate::AzureDevOpsServiceEndpointReference;
use crate::AzureDevOpsVariableGroupReference;
use arbitrary::Arbitrary;

/// Resources referenced by a YAML build process.
///
/// REST API context (7.1): the containing [BuildDefinition.process property](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition).
/// That REST page documents the base process but omits its YAML resources shape.
/// Supplemental JavaScript shape: [BuildProcessResources](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/buildprocessresources).
/// TypeScript source: [BuildProcessResources](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L1038).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildProcessResources {
    pub endpoints: Option<Vec<AzureDevOpsServiceEndpointReference>>,
    pub files: Option<Vec<AzureDevOpsSecureFileReference>>,
    pub queues: Option<Vec<AzureDevOpsAgentPoolQueueReference>>,
    pub variable_groups: Option<Vec<AzureDevOpsVariableGroupReference>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildProcessResources);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildProcessResources);
