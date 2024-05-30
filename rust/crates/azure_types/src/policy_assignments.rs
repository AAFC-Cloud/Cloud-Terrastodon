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

pub const POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicyAssignmentId {
    Unscoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ManagementGroupScoped { expanded: String },
}
impl PolicyAssignmentId {
    pub fn from_expanded_unscoped(expanded: &str) -> Result<Self> {
        let Some(name) = expanded.strip_prefix(POLICY_ASSIGNMENT_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!(
                "missing prefix, expected to begin with {POLICY_ASSIGNMENT_ID_PREFIX} and got {expanded}",
            ));
        };
        if !PolicyAssignmentId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(name.to_string());
        }
        Ok(PolicyAssignmentId::Unscoped {
            expanded: expanded.to_string(),
        })
    }

    pub fn from_expanded_subscription_scoped(expanded: &str) -> Result<Self> {
        let Some(remaining) = expanded.strip_prefix(SUBSCRIPTION_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
                .context(format!("missing subscription prefix, expected to begin with {SUBSCRIPTION_ID_PREFIX} and got {expanded}"));
        };
        let Some((_sub_name, remaining)) = remaining.split_once('/') else {
            return Err(ScopeError::Malformed).context(format!("bad name split given {expanded}"));
        };
        // Calculate the new slice that includes the slash using the original string's indices
        let remaining_with_slash = &expanded[expanded.len() - remaining.len() - 1..];
        let Some(name) = remaining_with_slash.strip_prefix(POLICY_ASSIGNMENT_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!("missing policy assignment prefix, expected to begin with {POLICY_ASSIGNMENT_ID_PREFIX} and got {remaining_with_slash} (full: {expanded})"));
        };
        if !PolicyAssignmentId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(expanded.to_string());
        }
        Ok(PolicyAssignmentId::SubscriptionScoped {
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
        let Some(name) = remaining_with_slash.strip_prefix(POLICY_ASSIGNMENT_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context(format!("missing policy assignment prefix, expected to begin with {POLICY_ASSIGNMENT_ID_PREFIX} and got {remaining_with_slash} (full: {expanded})"));
        };
        if !PolicyAssignmentId::is_valid_name(name) {
            return Err(ScopeError::InvalidName).context(expanded.to_string());
        }
        Ok(PolicyAssignmentId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        })
    }
    

    /// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftmanagement
    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 90 {
            return false;
        }

        // Must start with a letter or number
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_alphanumeric() {
                return false;
            }
        }

        // Cannot end with a period
        if name.ends_with('.') {
            return false;
        }

        // Allowed characters are alphanumerics, hyphens, underscores, periods, and parentheses
        name.chars()
            .all(|c| c.is_alphanumeric() || "-_().".contains(c))
    }
}

impl Scope for PolicyAssignmentId {

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
            PolicyAssignmentId::Unscoped { expanded } => expanded,
            PolicyAssignmentId::SubscriptionScoped { expanded } => expanded,
            PolicyAssignmentId::ManagementGroupScoped { expanded } => expanded,
        }
    }

    fn short_name(&self) -> &str {
        self.expanded_form()
            .strip_prefix(POLICY_ASSIGNMENT_ID_PREFIX)
            .unwrap_or_else(|| unreachable!("structure should have been validated at construction"))
    }
}

impl Serialize for PolicyAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyAssignmentId::from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyAssignment {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "enforcementMode")]
    pub enforcement_mode: String,
    pub id: PolicyAssignmentId,
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
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        f.write_fmt(format_args!("{:?}", &self.display_name))?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<PolicyAssignment> for TofuImportBlock {
    fn from(policy_assignment: PolicyAssignment) -> Self {
        TofuImportBlock {
            id: policy_assignment.id.expanded_form().to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::ManagementGroupPolicyAssignment,
                name: policy_assignment.name.sanitize(),
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
        let expanded = "/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicyAssignmentId::from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }
}
