use crate::scopes::Scope;
use crate::scopes::ScopeError;
use anyhow::Context;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

pub const MANAGEMENT_GROUP_ID_PREFIX: &str = "/providers/Microsoft.Management/managementGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupId {
    expanded: String,
}
impl ManagementGroupId {
    pub fn from_name(name: &str) -> Self {
        let expanded = format!("{}{}", MANAGEMENT_GROUP_ID_PREFIX, name);
        Self { expanded }
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

impl Scope for ManagementGroupId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        let Some(name) = expanded.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context("missing prefix");
        };
        if !ManagementGroupId::is_valid_name(name) {
            return Err(ScopeError::InvalidName.into());
        }
        Ok(ManagementGroupId {
            expanded: expanded.to_string(),
        })
    }

    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn short_form(&self) -> &str {
        self.expanded_form()
            .strip_prefix(MANAGEMENT_GROUP_ID_PREFIX)
            .unwrap_or_else(|| unreachable!("structure should have been validated at construction"))
    }
}

impl Serialize for ManagementGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for ManagementGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = ManagementGroupId::try_from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}

/// `az account management-group list --no-register --output json`
/// ```json
/// {
///   "displayName": "OPS",
///   "id": "/providers/Microsoft.Management/managementGroups/55555555-5555-5555-5555-555555555555",
///   "name": "55555555-5555-5555-5555-555555555555",  
///   "tenantId": "66666666-6666-6666-6666-666666666666",
///   "type": "Microsoft.Management/managementGroups"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ManagementGroup {
    #[serde(rename = "displayName")]
    pub display_name: String,
    // #[serde(deserialize_with = "deserialize_management_group_id")]
    pub id: ManagementGroupId,
    pub name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for ManagementGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}
