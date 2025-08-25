use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedRoleManagementPolicyAssignmentId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedRoleManagementPolicyAssignmentId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedRoleManagementPolicyAssignmentId;
use crate::prelude::RoleManagementPolicyAssignmentName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedRoleManagementPolicyAssignmentId;
use crate::prelude::UnscopedRoleManagementPolicyAssignmentId;
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
use uuid::Uuid;

pub const ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleManagementPolicyAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleManagementPolicyAssignmentId {
    Unscoped(UnscopedRoleManagementPolicyAssignmentId),
    ManagementGroupScoped(ManagementGroupScopedRoleManagementPolicyAssignmentId),
    SubscriptionScoped(SubscriptionScopedRoleManagementPolicyAssignmentId),
    ResourceGroupScoped(ResourceGroupScopedRoleManagementPolicyAssignmentId),
    ResourceScoped(ResourceScopedRoleManagementPolicyAssignmentId),
}
impl RoleManagementPolicyAssignmentId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleManagementPolicyAssignmentId::Unscoped(_) => None,
            RoleManagementPolicyAssignmentId::ManagementGroupScoped(_) => None,
            RoleManagementPolicyAssignmentId::SubscriptionScoped(
                subscription_scoped_role_assignment_id,
            ) => Some(*subscription_scoped_role_assignment_id.subscription_id()),
            RoleManagementPolicyAssignmentId::ResourceGroupScoped(
                resource_group_scoped_role_assignment_id,
            ) => Some(*resource_group_scoped_role_assignment_id.subscription_id()),
            RoleManagementPolicyAssignmentId::ResourceScoped(
                resource_scoped_role_assignment_id,
            ) => Some(*resource_scoped_role_assignment_id.subscription_id()),
        }
    }
}

// MARK: HasSlug
impl HasSlug for RoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        match self {
            RoleManagementPolicyAssignmentId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            RoleManagementPolicyAssignmentId::ManagementGroupScoped(
                management_group_scoped_role_assignment_id,
            ) => management_group_scoped_role_assignment_id.name(),
            RoleManagementPolicyAssignmentId::SubscriptionScoped(
                subscription_scoped_role_assignment_id,
            ) => subscription_scoped_role_assignment_id.name(),
            RoleManagementPolicyAssignmentId::ResourceGroupScoped(
                resource_group_scoped_role_assignment_id,
            ) => resource_group_scoped_role_assignment_id.name(),
            RoleManagementPolicyAssignmentId::ResourceScoped(
                resource_scoped_role_assignment_id,
            ) => resource_scoped_role_assignment_id.name(),
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleManagementPolicyAssignmentId::Unscoped(UnscopedRoleManagementPolicyAssignmentId {
            name,
        })
    }
}
impl TryFromResourceGroupScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyAssignmentId::ResourceGroupScoped(
            ResourceGroupScopedRoleManagementPolicyAssignmentId {
                resource_group_id,
                name,
            },
        )
    }
}
impl TryFromResourceScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyAssignmentId::ResourceScoped(
            ResourceScopedRoleManagementPolicyAssignmentId { resource_id, name },
        )
    }
}
impl TryFromSubscriptionScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyAssignmentId::SubscriptionScoped(
            SubscriptionScopedRoleManagementPolicyAssignmentId {
                subscription_id,
                name,
            },
        )
    }
}
impl TryFromManagementGroupScoped for RoleManagementPolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        RoleManagementPolicyAssignmentId::ManagementGroupScoped(
            ManagementGroupScopedRoleManagementPolicyAssignmentId {
                management_group_id,
                name,
            },
        )
    }
}

// MARK: impl Scope
impl Scope for RoleManagementPolicyAssignmentId {
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
        ScopeImplKind::RoleManagementPolicyAssignment
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for RoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for RoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}

// MARK: Serialize
impl Serialize for RoleManagementPolicyAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleManagementPolicyAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleManagementPolicyAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}
