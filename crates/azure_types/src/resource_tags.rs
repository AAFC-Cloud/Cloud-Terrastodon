use crate::prelude::HasScope;
use crate::prelude::Scope;
use crate::prelude::ScopeImpl;
use crate::prelude::ScopeImplKind;
use crate::prelude::strip_suffix_case_insensitive;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::str::FromStr;

pub const TAGS_SUFFIX: &str = "/providers/Microsoft.Resources/tags/default";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceTagsId {
    expanded: String,
}
impl ResourceTagsId {
    pub fn from_scope(resource: &impl HasScope) -> ResourceTagsId {
        match resource.scope().as_scope() {
            ScopeImpl::ResourceTags(x) => x,
            other => ResourceTagsId {
                expanded: format!("{}{}", other.expanded_form(), TAGS_SUFFIX),
            },
        }
    }
}
impl std::fmt::Display for ResourceTagsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded.to_string().as_str())
    }
}

impl FromStr for ResourceTagsId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        strip_suffix_case_insensitive(s, TAGS_SUFFIX).context(format!(
            "Id {} should end with {} to be a valid ResourceTagsId",
            s, TAGS_SUFFIX
        ))?;
        Ok(ResourceTagsId {
            expanded: s.to_string(),
        })
    }
}

impl Scope for ResourceTagsId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn short_form(&self) -> &str {
        let x = match strip_suffix_case_insensitive(&self.expanded, TAGS_SUFFIX) {
            Ok(x) => x,
            Err(_) => &self.expanded,
        };
        x.rsplit_once('/')
            .map(|x| x.1)
            .unwrap_or_else(|| self.expanded_form())
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        expanded.parse()
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::ResourceTags(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ResourceTags
    }
}

impl Serialize for ResourceTagsId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceTagsId {
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
