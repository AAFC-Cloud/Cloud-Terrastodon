use anyhow::Context;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::HashMap;
use tofu_types::imports::AzureRMResourceKind;
use tofu_types::imports::ImportBlock;
use tofu_types::imports::Sanitizable;
use tofu_types::imports::TofuResourceReference;
use anyhow::Result;

use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;
use crate::scopes::Scope;
use crate::scopes::ScopeError;

pub const POLICY_SET_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policySetDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PolicySetDefinitionId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
}
impl PolicySetDefinitionId {
    pub fn from_expanded_unscoped(expanded: &str) -> Result<Self> {
        let Some(name) = expanded.strip_prefix(POLICY_SET_DEFINITION_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!(
                "missing prefix, expected to begin with {POLICY_SET_DEFINITION_ID_PREFIX} and got {expanded}",
            ));
        };
        if !PolicySetDefinitionId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicySetDefinitionId::Unscoped {
            expanded: expanded.to_string(),
        })
    }

    pub fn from_expanded_management_group_scoped(expanded: &str) -> Result<Self> {
        let Some(remaining) = expanded.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
            .context(format!("missing management group prefix, expected to begin with {MANAGEMENT_GROUP_ID_PREFIX} and got {expanded}"));
        };
        let Some((_management_group_name, remaining)) = remaining.split_once("/") else {
            return Err(ScopeError::Malformed).context(format!("bad name split given {expanded}"));
        };
        // Calculate the new slice that includes the slash using the original string's indices
        let remaining_with_slash = &expanded[expanded.len() - remaining.len() - 1..];
        let Some(name) = remaining_with_slash.strip_prefix(POLICY_SET_DEFINITION_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!("missing policy assignment prefix, expected to begin with {POLICY_SET_DEFINITION_ID_PREFIX} and got {remaining_with_slash} (full: {expanded})"));
        };
        if !PolicySetDefinitionId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicySetDefinitionId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        })
    }

    pub fn from_name(name: &str) -> Self {
        let expanded = format!("{}{}", POLICY_SET_DEFINITION_ID_PREFIX, name);
        Self::Unscoped { expanded }
    }

    /// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
    fn is_valid_name(name: &str) -> bool {
        // Check the length constraints
        if name.len() < 1 || name.len() > 64 {
            return false;
        }

        // Define the set of forbidden characters
        let forbidden_chars = "<>*%&:\\?.+/";

        // Check for forbidden characters and control characters
        if name
            .chars()
            .any(|c| forbidden_chars.contains(c) || c.is_control())
        {
            return false;
        }

        // Check that it does not end with a period or a space
        if name.ends_with('.') || name.ends_with(' ') {
            return false;
        }

        true
    }
}

impl Scope for PolicySetDefinitionId {
    fn from_expanded(expanded: &str) -> Result<Self> {
        match Self::from_expanded_management_group_scoped(expanded) {
            Ok(x) => Ok(x),
            Err(e) => Self::from_expanded_unscoped(expanded).context(format!("tried management group scoped but it failed with {e:?}"))
        }
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
        }
    }
    
    fn short_name(&self) -> &str {
        self.expanded_form()
            .strip_prefix(POLICY_SET_DEFINITION_ID_PREFIX)
            .unwrap_or_else(|| unreachable!("structure should have been validated at construction"))
    }
}

impl Serialize for PolicySetDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicySetDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id =
            PolicySetDefinitionId::from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinitionPolicyDefinitionGroup {
    #[serde(rename = "additionalMetadataId")]
    pub additional_metadata_id: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinitionPolicyDefinition {
    #[serde(rename = "groupNames")]
    pub group_names: Option<Vec<String>>,
    pub parameters: Value,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    #[serde(rename = "policyDefinitionReferenceId")]
    pub policy_definition_reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinition {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: PolicySetDefinitionId,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyDefinitionGroups")]
    pub policy_definition_groups: Option<Vec<PolicySetDefinitionPolicyDefinitionGroup>>,
    #[serde(rename = "policyDefinitions")]
    pub policy_definitions: Option<Vec<PolicySetDefinitionPolicyDefinition>>,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicySetDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<PolicySetDefinition> for ImportBlock {
    fn from(policy_definition: PolicySetDefinition) -> Self {
        ImportBlock {
            id: policy_definition.id.expanded_form().to_string(),
            to: TofuResourceReference::AzureRM {
                kind: AzureRMResourceKind::PolicySetDefinition,
                name: policy_definition.display_name.sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }
    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicySetDefinitionId::from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }
    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id: PolicySetDefinitionId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.expanded_form(), expanded);

        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id: PolicySetDefinitionId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.expanded_form(), expanded);

        Ok(())
    }
}
