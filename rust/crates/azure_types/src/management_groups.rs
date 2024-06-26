use crate::resource_name_rules::validate_management_group_name;
use crate::scopes::Scope;
use crate::scopes::ScopeError;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use anyhow::Context;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::hash::Hash;

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
}

impl Scope for ManagementGroupId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        let Some(name) = expanded.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX) else {
            return Err(ScopeError::Malformed).context("missing prefix");
        };
        validate_management_group_name(name)?;
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

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ManagementGroup
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ManagementGroup(self.clone())
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
        let id =
            ManagementGroupId::try_from_expanded(expanded.as_str()).map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagementGroup {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: ManagementGroupId,
    pub name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for ManagementGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        f.write_str(&self.display_name)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl Hash for ManagementGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ManagementGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ManagementGroup {}
