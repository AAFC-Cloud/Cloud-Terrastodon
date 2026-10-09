use crate::AzureDevOpsBuildAgentSpecification;
use arbitrary::Arbitrary;

/// The execution target of a designer or Docker build process.
///
/// REST API context (7.1): the containing [BuildDefinition.process property](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition).
/// That REST page documents the base process but omits derived target fields.
/// Supplemental JavaScript shape: [DesignerProcessTarget](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/designerprocesstarget).
/// TypeScript source: [DesignerProcessTarget](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L1821).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDesignerProcessTarget {
    pub agent_specification: Option<AzureDevOpsBuildAgentSpecification>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDesignerProcessTarget);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDesignerProcessTarget);
