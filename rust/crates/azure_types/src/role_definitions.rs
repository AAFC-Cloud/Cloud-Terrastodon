use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use anyhow::bail;
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::pattern::Pattern;
use std::str::FromStr;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderReference;
use tofu_types::prelude::TofuResourceReference;

pub const ROLE_DEFINITION_ID_PREFIX: &str = "/providers/Microsoft.Authorization/RoleDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RoleDefinitionId {
    expanded: String,
}

impl HasPrefix for RoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}

impl std::fmt::Display for RoleDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded_form())
    }
}

impl FromStr for RoleDefinitionId {
    type Err = anyhow::Error;

    fn from_str(expanded: &str) -> Result<Self, Self::Err> {
        if !ROLE_DEFINITION_ID_PREFIX.is_prefix_of(expanded) {
            bail!(
                "Missing prefix {ROLE_DEFINITION_ID_PREFIX} trying to parse {expanded} as {:?}",
                ScopeImplKind::RoleDefinition
            );
        }

        Ok(RoleDefinitionId {
            expanded: expanded.to_owned(),
        })
    }
}

impl Scope for RoleDefinitionId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn short_form(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .expect("no slash found, structure should have been validated at construction")
            .1
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        expanded.parse()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleDefinition(self.clone())
    }
}

impl Serialize for RoleDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleDefinitionId {
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
pub struct RolePermission {
    #[serde(rename = "notDataActions")]
    not_data_actions: Vec<String>,
    #[serde(rename = "dataActions")]
    data_actions: Vec<String>,
    #[serde(rename = "notActions")]
    not_actions: Vec<String>,
    #[serde(rename = "actions")]
    actions: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RoleDefinitionKind {
    BuiltInRole,
    CustomRole
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleDefinition {
    pub id: RoleDefinitionId,
    pub display_name: String,
    pub description: String,
    pub assignable_scopes: Vec<String>,
    pub permissions: Vec<RolePermission>,
    pub kind: RoleDefinitionKind,
}

impl HasScope for RoleDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &RoleDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for RoleDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(self.id.short_form())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<RoleDefinition> for TofuImportBlock {
    fn from(resource_group: RoleDefinition) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: resource_group.id.to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::RoleDefinition,
                name: format!("{}__{}", resource_group.display_name, resource_group.id.short_form()).sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = format!("{}{}", ROLE_DEFINITION_ID_PREFIX, Uuid::default());
        let id: RoleDefinitionId = serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
