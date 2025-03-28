use crate::prelude::HasScope;
use crate::prelude::ResourceType;
use crate::prelude::Scope;
use crate::prelude::ScopeImpl;
use crate::prelude::ScopeImplKind;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::collections::HashMap;
use std::str::FromStr;
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceId {
    expanded: String,
}
impl ResourceId {
    pub fn new(arg: impl Into<String>) -> Self {
        Self {
            expanded: arg.into(),
        }
    }
}
impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded.to_string().as_str())
    }
}

impl FromStr for ResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ResourceId {
            expanded: s.to_string(),
        })
    }
}

impl Scope for ResourceId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        Ok(ResourceId {
            expanded: expanded.to_owned(),
        })
    }

    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::Raw(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Raw
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Resource {
    pub id: ResourceId,
    pub kind: ResourceType,
    pub name: String,
    pub display_name: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}
impl HasScope for Resource {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "display_name={:64?}\tname={:64?}\tid={}",
            self.display_name, self.name, self.id
        ))
    }
}
