use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedRoleManagementPolicyId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedRoleManagementPolicyId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedRoleManagementPolicyId;
use crate::prelude::RoleManagementPolicyAssignmentName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedRoleManagementPolicyId;
use crate::prelude::UnscopedRoleManagementPolicyId;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromResourceScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use crate::scopes::try_from_expanded_hierarchy_scoped;
use crate::slug::HasSlug;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use uuid::Uuid;


pub const ROLE_MANAGEMENT_POLICY_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleManagementPolicies/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleManagementPolicyId {
    Unscoped(UnscopedRoleManagementPolicyId),
    ManagementGroupScoped(ManagementGroupScopedRoleManagementPolicyId),
    SubscriptionScoped(SubscriptionScopedRoleManagementPolicyId),
    ResourceGroupScoped(ResourceGroupScopedRoleManagementPolicyId),
    ResourceScoped(ResourceScopedRoleManagementPolicyId),
}
impl RoleManagementPolicyId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleManagementPolicyId::Unscoped(_) => None,
            RoleManagementPolicyId::ManagementGroupScoped(_) => None,
            RoleManagementPolicyId::SubscriptionScoped(subscription_scoped_role_assignment_id) => Some(
                *subscription_scoped_role_assignment_id
                    .subscription_id(),
            ),
            RoleManagementPolicyId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(
                    *resource_group_scoped_role_assignment_id
                        .subscription_id(),
                )
            }
            RoleManagementPolicyId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(*resource_scoped_role_assignment_id.subscription_id())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for RoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        match self {
            RoleManagementPolicyId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            RoleManagementPolicyId::ManagementGroupScoped(management_group_scoped_role_assignment_id) => {
                management_group_scoped_role_assignment_id.name()
            }
            RoleManagementPolicyId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                subscription_scoped_role_assignment_id.name()
            }
            RoleManagementPolicyId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                resource_group_scoped_role_assignment_id.name()
            }
            RoleManagementPolicyId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for RoleManagementPolicyId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleManagementPolicyId::Unscoped(UnscopedRoleManagementPolicyId {
            name,
        })
    }
}
impl TryFromResourceGroupScoped for RoleManagementPolicyId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyId::ResourceGroupScoped(ResourceGroupScopedRoleManagementPolicyId {
            resource_group_id,
            name,
        })
    }
}
impl TryFromResourceScoped for RoleManagementPolicyId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyId::ResourceScoped(ResourceScopedRoleManagementPolicyId {
            resource_id,
            name,
        })
    }
}
impl TryFromSubscriptionScoped for RoleManagementPolicyId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyId::SubscriptionScoped(SubscriptionScopedRoleManagementPolicyId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for RoleManagementPolicyId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyId::ManagementGroupScoped(ManagementGroupScopedRoleManagementPolicyId {
            management_group_id,
            name,
        })
    }
}


// MARK: impl Scope
impl Scope for RoleManagementPolicyId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        match self {
            Self::Unscoped(x) => x.expanded_form(),
            Self::ResourceGroupScoped(x) => x.expanded_form(),
            Self::SubscriptionScoped(x) => x.expanded_form(),
            Self::ManagementGroupScoped(x) => x.expanded_form(),
            Self::ResourceScoped(x) => x.expanded_form(),
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicy
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleManagementPolicy(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for RoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for RoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}



// MARK: Serialize
impl Serialize for RoleManagementPolicyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleManagementPolicyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleManagementPolicyId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
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
