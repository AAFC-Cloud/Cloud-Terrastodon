use crate::resource_name_rules::validate_resource_group_name;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromSubscriptionScoped;
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
use tofu_types::prelude::TofuProviderReference;
use tofu_types::prelude::TofuResourceReference;

pub const RESOURCE_GROUP_ID_PREFIX: &str = "/resourceGroups/";
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupId {
    expanded: String,
}

impl std::fmt::Display for ResourceGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded.to_string().as_str())
    }
}

impl NameValidatable for ResourceGroupId {
    fn validate_name(name: &str) -> Result<()> {
        validate_resource_group_name(name)
    }
}
impl HasPrefix for ResourceGroupId {
    fn get_prefix() -> &'static str {
        RESOURCE_GROUP_ID_PREFIX
    }
}
impl TryFromSubscriptionScoped for ResourceGroupId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        ResourceGroupId {
            expanded: expanded.to_owned(),
        }
    }
}

impl FromStr for ResourceGroupId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ResourceGroupId {
            expanded: s.to_string(),
        })
    }
}

impl Scope for ResourceGroupId {
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
        ResourceGroupId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ResourceGroup(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ResourceGroup
    }
}

impl Serialize for ResourceGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded.parse().map_err(|e| D::Error::custom(format!("{e:#}")))?;
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
}

impl HasScope for ResourceGroup {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &ResourceGroup {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ResourceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl From<ResourceGroup> for TofuImportBlock {
    fn from(resource_group: ResourceGroup) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
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
