use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::HashMap;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuResourceReference;

use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::scopes::Scope;
use crate::scopes::ScopeError;

pub const POLICY_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicyDefinitionId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
}
impl PolicyDefinitionId {
    // pub fn name(&self) -> &str {}
    pub fn from_expanded_unscoped(expanded: &str) -> Result<Self> {
        let Some(name) = expanded.strip_prefix(POLICY_DEFINITION_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!(
                "missing prefix, expected to begin with {POLICY_DEFINITION_ID_PREFIX} and got {expanded}",
            ));
        };
        if !PolicyDefinitionId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicyDefinitionId::Unscoped {
            expanded: expanded.to_string(),
        })
    }

    pub fn from_expanded_management_group_scoped(expanded: &str) -> Result<Self> {
        let Some(remaining) = expanded.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
            .context(format!("missing management group prefix, expected to begin with {MANAGEMENT_GROUP_ID_PREFIX} and got {expanded}"));
        };
        let Some((_management_group_name, remaining)) = remaining.split_once('/') else {
            return Err(ScopeError::Malformed).context(format!("bad name split given {expanded}"));
        };
        // Calculate the new slice that includes the slash using the original string's indices
        let remaining_with_slash = &expanded[expanded.len() - remaining.len() - 1..];
        let Some(name) = remaining_with_slash.strip_prefix(POLICY_DEFINITION_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!("missing policy assignment prefix, expected to begin with {POLICY_DEFINITION_ID_PREFIX} and got {remaining_with_slash} (full: {expanded})"));
        };
        if !PolicyDefinitionId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicyDefinitionId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        })
    }

    pub fn from_expanded_subscription_scoped(expanded: &str) -> Result<Self> {
        let Some(remaining) = expanded.strip_prefix(SUBSCRIPTION_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
            .context(format!("missing subscription prefix, expected to begin with {SUBSCRIPTION_ID_PREFIX} and got {expanded}"));
        };
        let Some((_subscription_id, remaining)) = remaining.split_once('/') else {
            return Err(ScopeError::Malformed).context(format!("bad name split given {expanded}"));
        };
        // Calculate the new slice that includes the slash using the original string's indices
        let remaining_with_slash = &expanded[expanded.len() - remaining.len() - 1..];
        let Some(name) = remaining_with_slash.strip_prefix(POLICY_DEFINITION_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!("missing policy assignment prefix, expected to begin with {POLICY_DEFINITION_ID_PREFIX} and got {remaining_with_slash} (full: {expanded})"));
        };
        if !PolicyDefinitionId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicyDefinitionId::SubscriptionScoped {
            expanded: expanded.to_string(),
        })
    }

    /// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
    fn is_valid_name(name: &str) -> bool {
        // Check the length constraints
        if name.is_empty() || name.len() > 64 {
            return false;
        }

        // Define the set of forbidden characters
        // Periods are listed as forbidden on the website but we have some in our tenant :/
        // https://github.com/MicrosoftDocs/azure-docs/issues/122020
        // let forbidden_chars = "<>*%&:\\?.+/";
        let forbidden_chars = "<>*%&:\\?+/";

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

impl Scope for PolicyDefinitionId {
    fn from_expanded(expanded: &str) -> Result<Self> {
        match Self::from_expanded_management_group_scoped(expanded) {
            Ok(x) => Ok(x),
            Err(management_group_scoped_error) => {
                match Self::from_expanded_subscription_scoped(expanded) {
                    Ok(x) => Ok(x),
                    Err(subscription_scoped_error) => {
                        match Self::from_expanded_unscoped(expanded) {
                            Ok(x) => Ok(x),
                            Err(unscoped_error) => {
                                bail!("Policy definition id parse failed.\nmanagement group scoped attempt: {management_group_scoped_error:?}\nsubscription scoped attempt: {subscription_scoped_error:?}\nunscoped attempt: {unscoped_error:?}")
                            }
                        }
                    }
                }
            }
        }
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
            Self::SubscriptionScoped { expanded } => expanded,
        }
    }

    fn short_name(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .expect("no slash found, form should have been validated at construction")
            .1
    }
}

impl Serialize for PolicyDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyDefinitionId::from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: PolicyDefinitionId,
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
impl From<PolicyDefinition> for TofuImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        TofuImportBlock {
            id: policy_definition.id.expanded_form().to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::PolicyDefinition,
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
        let expanded = "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyDefinitionId::Unscoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_name(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyDefinitionId::ManagementGroupScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_name(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id: PolicyDefinitionId =
            serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.expanded_form(), expanded);

        Ok(())
    }
}
