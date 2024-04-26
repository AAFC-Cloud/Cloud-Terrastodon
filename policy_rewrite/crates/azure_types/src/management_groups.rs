use crate::scopes::Scope;
use crate::scopes::ScopeError;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

const PREFIX: &str = "/providers/Microsoft.Management/managementGroups/";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ManagementGroupId {
    expanded: String,
}
impl ManagementGroupId {
    pub fn new(name: &str) -> Self {
        let expanded = format!("{}{}", PREFIX, name);
        Self { expanded }
    }
    pub fn from_expanded(expanded: &str) -> Result<Self, ScopeError> {
        if expanded.starts_with(PREFIX) {
            let name = &expanded[PREFIX.len()..];

            // Check the name is valid based on Microsoft's documentation
            if !ManagementGroupId::is_valid_name(name) {
                return Err(ScopeError::InvalidName);
            }

            Ok(ManagementGroupId {
                expanded: expanded.to_string(),
            })
        } else {
            Err(ScopeError::Malformed)
        }
    }

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

impl TryFrom<&str> for ManagementGroupId {
    type Error = ScopeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_expanded(value)
    }
}

impl Scope for ManagementGroupId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn short_name(&self) -> &str {
        self.expanded
            .strip_prefix(PREFIX)
            .unwrap_or_else(|| unreachable!("structure should have been validated at construction"))
    }
}

// fn deserialize_management_group_id<'de, D>(deserializer: D) -> Result<ManagementGroupId, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let id_str = String::deserialize(deserializer)?;
//     Ok(ManagementGroupId::from_expanded(id_str))
// }

impl Serialize for ManagementGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded)
    }
}

impl<'de> Deserialize<'de> for ManagementGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = ManagementGroupId::from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}
/// `az account management-group list --no-register --output json`
/// ```
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