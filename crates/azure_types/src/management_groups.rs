use crate::naming::validate_management_group_name;
use crate::prelude::HasPrefix;
use crate::prelude::ManagementGroupAncestorsChain;
use crate::prelude::NameValidatable;
use crate::prelude::TenantId;
use crate::prelude::strip_prefix_case_insensitive;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::hash::Hash;
use std::str::FromStr;

pub const MANAGEMENT_GROUP_ID_PREFIX: &str = "/providers/Microsoft.Management/managementGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupId {
    expanded: String,
}
impl std::fmt::Display for ManagementGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expanded)
    }
}
impl ManagementGroupId {
    pub fn from_name(name: &str) -> Self {
        let expanded = format!("{MANAGEMENT_GROUP_ID_PREFIX}{name}");
        Self { expanded }
    }
    pub fn name(&self) -> &str {
        &self.expanded[MANAGEMENT_GROUP_ID_PREFIX.len()..]
    }
}
impl HasPrefix for ManagementGroupId {
    fn get_prefix() -> &'static str {
        MANAGEMENT_GROUP_ID_PREFIX
    }
}

impl FromStr for ManagementGroupId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX).unwrap_or(s);
        Ok(Self::from_name(s))
    }
}
impl NameValidatable for ManagementGroupId {
    fn validate_name(name: &str) -> Result<()> {
        validate_management_group_name(name)
    }
}
impl Scope for ManagementGroupId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        // this doesn't use TryFromManagementGroupScoped because it itself is the scope, the management group isn't a prefix
        let name = strip_prefix_case_insensitive(expanded, MANAGEMENT_GROUP_ID_PREFIX)?;
        validate_management_group_name(name)?;
        Ok(ManagementGroupId {
            expanded: expanded.to_string(),
        })
    }

    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ManagementGroup
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ManagementGroup(self.clone())
    }
}

impl Serialize for ManagementGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for ManagementGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = ManagementGroupId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagementGroup {
    pub display_name: String,
    pub id: ManagementGroupId,
    pub tenant_id: TenantId,
    pub management_group_ancestors_chain: ManagementGroupAncestorsChain,
}
impl ManagementGroup {
    pub fn name(&self) -> &str {
        self.id.name()
    }
}
impl AsScope for ManagementGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &ManagementGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl std::fmt::Display for ManagementGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_test() -> Result<()> {
        let id = ManagementGroupId::from_name("bruh");
        let name = id.name();
        assert_eq!(name, "bruh");
        Ok(())
    }
}
