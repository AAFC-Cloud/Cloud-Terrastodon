use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tofu_types::imports::AzureRMResourceKind;
use tofu_types::imports::ImportBlock;
use tofu_types::imports::Sanitizable;
use tofu_types::imports::TofuResourceReference;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiativePolicyDefinitionGroup {
    #[serde(rename = "additionalMetadataId")]
    pub additional_metadata_id: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiativePolicyDefinition {
    #[serde(rename = "groupNames")]
    pub group_names: Option<Vec<String>>,
    pub parameters: Value,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    #[serde(rename = "policyDefinitionReferenceId")]
    pub policy_definition_reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiative {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyDefinitionGroups")]
    pub policy_definition_groups: Option<Vec<PolicyInitiativePolicyDefinitionGroup>>,
    #[serde(rename = "policyDefinitions")]
    pub policy_definitions: Option<Vec<PolicyInitiativePolicyDefinition>>,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyInitiative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<PolicyInitiative> for ImportBlock {
    fn from(policy_definition: PolicyInitiative) -> Self {
        ImportBlock {
            id: policy_definition.id.clone(),
            to: TofuResourceReference::AzureRM {
                kind: AzureRMResourceKind::PolicySetDefinition,
                name: policy_definition.display_name.sanitize(),
            },
        }
    }
}
