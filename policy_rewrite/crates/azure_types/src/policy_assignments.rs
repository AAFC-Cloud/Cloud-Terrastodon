use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tofu_types::imports::AzureRMResourceKind;
use tofu_types::imports::ImportBlock;
use tofu_types::imports::Sanitizable;
use tofu_types::imports::TofuResourceReference;
use std::collections::HashMap;
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyAssignment {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "enforcementMode")]
    pub enforcement_mode: String,
    pub id: String,
    pub identity: Option<HashMap<String, Value>>,
    pub location: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    #[serde(rename = "nonComplianceMessages")]
    pub non_compliance_messages: Option<Value>,
    #[serde(rename = "notScopes")]
    pub not_scopes: Option<Vec<String>>,
    pub parameters: Option<Value>,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    pub scope: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<PolicyAssignment> for ImportBlock {
    fn from(policy_assignment: PolicyAssignment) -> Self {
        ImportBlock {
            id: policy_assignment.id.clone(),
            to: TofuResourceReference::AzureRM {
                kind: AzureRMResourceKind::ManagementGroupPolicyAssignment,
                name: policy_assignment.display_name.sanitize(),
            },
        }
    }
}