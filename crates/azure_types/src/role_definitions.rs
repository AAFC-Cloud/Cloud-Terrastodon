use crate::prelude::Fake;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::strip_prefix_case_insensitive;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::str::FromStr;
use uuid::Uuid;

pub const ROLE_DEFINITION_ID_PREFIX: &str = "/providers/Microsoft.Authorization/RoleDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RoleDefinitionId {
    expanded: String,
}

// TODO: shouldn't this be NameValidatable as a guid?
impl RoleDefinitionId {
    pub fn new(uuid: &Uuid) -> Self {
        RoleDefinitionId {
            expanded: format!("{ROLE_DEFINITION_ID_PREFIX}{}", uuid.as_hyphenated()),
        }
    }
}
impl Fake for RoleDefinitionId {
    fn fake() -> Self {
        RoleDefinitionId::new(&Uuid::nil())
    }
}

impl HasPrefix for RoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}

impl std::fmt::Display for RoleDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expanded_form())
    }
}

impl FromStr for RoleDefinitionId {
    type Err = eyre::Error;

    fn from_str(expanded: &str) -> Result<Self, Self::Err> {
        if strip_prefix_case_insensitive(expanded, ROLE_DEFINITION_ID_PREFIX).is_err() {
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
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
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
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:#?}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RolePermission {
    #[serde(rename = "notDataActions")]
    #[serde(alias = "NotDataActions")]
    not_data_actions: Vec<String>,
    #[serde(rename = "dataActions")]
    #[serde(alias = "DataActions")]
    data_actions: Vec<String>,
    #[serde(rename = "notActions")]
    #[serde(alias = "NotActions")]
    not_actions: Vec<String>,
    #[serde(rename = "actions")]
    #[serde(alias = "Actions")]
    actions: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum RoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RoleDefinition {
    pub id: RoleDefinitionId,
    pub display_name: String,
    pub description: String,
    pub assignable_scopes: Vec<String>,
    pub permissions: Vec<RolePermission>,
    pub kind: RoleDefinitionKind,
}

impl Fake for RoleDefinition {
    fn fake() -> Self {
        RoleDefinition {
            id: Fake::fake(),
            display_name: "Fake Role".to_owned(),
            description: "Fake role description".to_owned(),
            assignable_scopes: vec![],
            permissions: vec![],
            kind: RoleDefinitionKind::CustomRole,
        }
    }
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
        f.write_str(&self.id.short_form())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<RoleDefinition> for HCLImportBlock {
    fn from(role_definition: RoleDefinition) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: role_definition.id.to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::RoleDefinition,
                name: format!(
                    "{}__{}",
                    role_definition.display_name,
                    role_definition.id.short_form()
                )
                .sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = format!("{}{}", ROLE_DEFINITION_ID_PREFIX, Uuid::default());
        let id: RoleDefinitionId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
