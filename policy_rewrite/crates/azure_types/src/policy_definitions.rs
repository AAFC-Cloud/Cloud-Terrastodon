use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tofu_types::imports::AzureRMResourceKind;
use tofu_types::imports::ImportBlock;
use tofu_types::imports::Sanitizable;
use tofu_types::imports::TofuResourceReference;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    pub metadata: HashMap<String, Value>,
    pub mode: String,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyRule")]
    pub policy_rule: serde_json::Value,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<PolicyDefinition> for ImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        ImportBlock {
            id: policy_definition.id.clone(),
            to: TofuResourceReference::AzureRM {
                kind: AzureRMResourceKind::PolicyDefinition,
                name: policy_definition.display_name.sanitize(),
            },
        }
    }
}
