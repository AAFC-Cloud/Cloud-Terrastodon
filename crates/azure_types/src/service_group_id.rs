use crate::prelude::HasPrefix;
use crate::prelude::NameValidatable;
use crate::prelude::ServiceGroupName;
use crate::prelude::strip_prefix_case_insensitive;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::slug::Slug;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::hash::Hash;
use std::str::FromStr;

pub const SERVICE_GROUP_ID_PREFIX: &str = "/providers/Microsoft.Management/serviceGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServiceGroupId {
    name: ServiceGroupName,
}

impl ServiceGroupId {
    pub fn from_name(name: ServiceGroupName) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &ServiceGroupName {
        &self.name
    }
}

impl std::fmt::Display for ServiceGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expanded_form())
    }
}

impl HasPrefix for ServiceGroupId {
    fn get_prefix() -> &'static str {
        SERVICE_GROUP_ID_PREFIX
    }
}

impl NameValidatable for ServiceGroupId {
    fn validate_name(name: &str) -> Result<()> {
        ServiceGroupName::try_new(name)?;
        Ok(())
    }
}

impl FromStr for ServiceGroupId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ServiceGroupId::try_from_expanded(s)
    }
}

impl Scope for ServiceGroupId {
    type Err = eyre::Error;

    fn expanded_form(&self) -> String {
        format!("{SERVICE_GROUP_ID_PREFIX}{}", self.name)
    }

    fn try_from_expanded(expanded: &str) -> Result<Self, <Self as Scope>::Err> {
        let name = strip_prefix_case_insensitive(expanded, SERVICE_GROUP_ID_PREFIX)?;
        let name = ServiceGroupName::try_new(name)?;
        Ok(Self { name })
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ServiceGroup
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::ServiceGroup(self.clone())
    }
}

impl Serialize for ServiceGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for ServiceGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        ServiceGroupId::try_from_expanded(&expanded)
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scopes::Scope;

    #[test]
    fn round_trip() -> Result<()> {
        let name = ServiceGroupName::try_new("MyServiceGroup")?;
        let id = ServiceGroupId::from_name(name.clone());
        assert_eq!(id.name(), &name);
        assert_eq!(ServiceGroupId::try_from_expanded(&id.expanded_form())?, id);
        Ok(())
    }
}
