use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use tofu_types::prelude::TofuProviderKind;
use tofu_types::prelude::TofuProviderReference;
use std::str::FromStr;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuResourceReference;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RoleAssignmentId(String);
impl RoleAssignmentId {
    pub fn expanded_form(&self) -> &str {
        &self.0
    }
    pub fn short_name(&self) -> &str {
        todo!()
    }
}

impl std::fmt::Display for RoleAssignmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl FromStr for RoleAssignmentId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RoleAssignmentId(s.to_string()))
    }
}

impl Serialize for RoleAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for RoleAssignmentId {
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
pub struct RoleAssignment {
    condition: Option<Value>,
    #[serde(rename = "conditionVersion")]
    condition_version: Option<Value>,
    #[serde(rename = "createdBy")]
    created_by: Uuid,
    #[serde(rename = "createdOn")]
    created_on: DateTime<Utc>,
    #[serde(rename = "delegatedManagedIdentityResourceId")]
    delegated_managed_identity_resource_id: Option<Value>,
    description: Option<Value>,
    id: String,
    name: Uuid,
    #[serde(rename = "principalId")]
    principal_id: Uuid,
    #[serde(rename = "principalName")]
    principal_name: String,
    #[serde(rename = "principalType")]
    principal_type: String,
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: String,
    #[serde(rename = "roleDefinitionName")]
    role_definition_name: String,
    scope: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "updatedBy")]
    updated_by: Uuid,
    #[serde(rename = "updatedOn")]
    updated_on: DateTime<Utc>,
}
impl std::fmt::Display for RoleAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.role_definition_name)?;
        f.write_str(" for ")?;
        f.write_str(&self.principal_name)?;
        f.write_str(" (")?;
        f.write_str(self.principal_id.to_string().as_str())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<RoleAssignment> for TofuImportBlock {
    fn from(resource_group: RoleAssignment) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Default { kind: Some(TofuProviderKind::AzureRM) },
            id: resource_group.id.to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::RoleAssignment,
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
        let id: RoleAssignmentId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
