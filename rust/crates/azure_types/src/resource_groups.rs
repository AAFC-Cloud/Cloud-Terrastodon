use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;
use std::str::FromStr;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuResourceReference;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupId(String);
impl ResourceGroupId {
    pub fn expanded_form(&self) -> &str {
        &self.0
    }
    pub fn short_name(&self) -> &str {
        todo!()
    }
}

impl std::fmt::Display for ResourceGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl FromStr for ResourceGroupId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ok(ResourceGroupId(uuid::Uuid::parse_str(s)?))
        Ok(ResourceGroupId(s.to_string()))
    }
}

impl Serialize for ResourceGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded.parse().map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResourceGroup {
    pub id: ResourceGroupId,
    pub location: String,
    #[serde(rename = "managedBy")]
    pub managed_by: Option<String>,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub tags: Option<HashMap<String, String>>,
    #[serde(rename = "type")]
    pub kind: String,
    // description: Option<String>,
    // #[serde(rename = "displayName")]
    // pub display_name: String,
    // pub id: ResourceGroupId,
    // #[serde(rename = "isAssignableToRole")]
    // pub is_assignable_to_role: Option<bool>,
    // #[serde(rename = "securityEnabled")]
    // pub security_enabled: bool,
}
impl std::fmt::Display for ResourceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<ResourceGroup> for TofuImportBlock {
    fn from(resource_group: ResourceGroup) -> Self {
        TofuImportBlock {
            id: resource_group.id.to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::ResourceGroup,
                name: format!("{}__{}", resource_group.name, resource_group.id).sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: ResourceGroupId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
