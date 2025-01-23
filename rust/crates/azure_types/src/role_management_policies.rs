use crate::scopes::try_from_expanded_resource_container_scoped;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use eyre::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use uuid::Uuid;

pub const ROLE_MANAGEMENT_POLICY_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleManagementPolicies/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleManagementPolicyId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
}
impl NameValidatable for RoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for RoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
impl TryFromUnscoped for RoleManagementPolicyId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for RoleManagementPolicyId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromSubscriptionScoped for RoleManagementPolicyId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromManagementGroupScoped for RoleManagementPolicyId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleManagementPolicyId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for RoleManagementPolicyId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_resource_container_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ResourceGroupScoped { expanded } => expanded,
            Self::SubscriptionScoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicy
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleManagementPolicy(self.clone())
    }
}

impl Serialize for RoleManagementPolicyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleManagementPolicyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleManagementPolicyId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;

    use super::*;
    #[test]
    fn it_works() -> Result<()> {
        let id = format!(
            "{}{}{}{}",
            MANAGEMENT_GROUP_ID_PREFIX,
            Uuid::nil(),
            ROLE_MANAGEMENT_POLICY_ID_PREFIX,
            Uuid::nil(),
        );
        RoleManagementPolicyId::try_from_expanded(&id)?;
        Ok(())
    }
}
