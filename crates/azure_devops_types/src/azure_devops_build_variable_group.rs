use crate::AzureDevOpsBuildDefinitionVariable;
use crate::AzureDevOpsVariableGroupId;
use crate::AzureDevOpsVariableGroupName;
use arbitrary::Arbitrary;
use std::collections::BTreeMap;

/// A variable group included in a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#variablegroup).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildVariableGroup {
    pub id: AzureDevOpsVariableGroupId,
    pub alias: Option<String>,
    pub name: Option<AzureDevOpsVariableGroupName>,
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub variables: Option<BTreeMap<String, AzureDevOpsBuildDefinitionVariable>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildVariableGroup);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildVariableGroup);
