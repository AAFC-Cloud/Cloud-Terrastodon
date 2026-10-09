use crate::AzureDevOpsBuildDesignerProcessTarget;
use crate::AzureDevOpsBuildProcessResources;
use crate::AzureDevOpsBuildProcessType;
use crate::AzureDevOpsBuildYamlFilename;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;

/// A build process with the properties of its published derived response shapes.
///
/// REST API schema (7.1): [BuildProcess](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildprocess).
/// It documents only `type`; the derived YAML and designer fields are omitted.
/// Supplemental JavaScript shapes: [YamlProcess](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/yamlprocess)
/// and [DesignerProcess](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/designerprocess).
/// TypeScript source: [BuildProcess](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L1028).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildProcess {
    pub r#type: AzureDevOpsBuildProcessType,
    /// Only YAML processes have a repository YAML filename.
    pub yaml_filename: Option<AzureDevOpsBuildYamlFilename>,
    pub errors: Option<Vec<String>>,
    pub resources: Option<AzureDevOpsBuildProcessResources>,
    /// Designer phases contain task- and execution-target-specific graphs.
    /// This read-only definition model preserves that graph as an explicit
    /// opaque subtree; it does not expose a designer-phase editing contract.
    pub phases: Option<Vec<ArbitraryJson>>,
    pub target: Option<AzureDevOpsBuildDesignerProcessTarget>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildProcess);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildProcess);
