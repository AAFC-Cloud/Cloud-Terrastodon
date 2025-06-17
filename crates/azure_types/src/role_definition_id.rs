use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedRoleDefinitionId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedRoleDefinitionId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedRoleDefinitionId;
use crate::prelude::RoleDefinitionName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedRoleDefinitionId;
use crate::prelude::UnscopedRoleDefinitionId;
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

pub const ROLE_DEFINITION_ID_PREFIX: &str = "/providers/Microsoft.Authorization/RoleDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleDefinitionId {
    Unscoped(UnscopedRoleDefinitionId),
    ManagementGroupScoped(ManagementGroupScopedRoleDefinitionId),
    SubscriptionScoped(SubscriptionScopedRoleDefinitionId),
    ResourceGroupScoped(ResourceGroupScopedRoleDefinitionId),
    ResourceScoped(ResourceScopedRoleDefinitionId),
}
impl RoleDefinitionId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleDefinitionId::Unscoped(_) => None,
            RoleDefinitionId::ManagementGroupScoped(_) => None,
            RoleDefinitionId::SubscriptionScoped(subscription_scoped_role_definition_id) => {
                Some(*subscription_scoped_role_definition_id.subscription_id())
            }
            RoleDefinitionId::ResourceGroupScoped(resource_group_scoped_role_definition_id) => {
                Some(*resource_group_scoped_role_definition_id.subscription_id())
            }
            RoleDefinitionId::ResourceScoped(resource_scoped_role_definition_id) => {
                Some(*resource_scoped_role_definition_id.subscription_id())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for RoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        match self {
            RoleDefinitionId::Unscoped(unscoped_role_definition_id) => {
                unscoped_role_definition_id.name()
            }
            RoleDefinitionId::ManagementGroupScoped(management_group_scoped_role_definition_id) => {
                management_group_scoped_role_definition_id.name()
            }
            RoleDefinitionId::SubscriptionScoped(subscription_scoped_role_definition_id) => {
                subscription_scoped_role_definition_id.name()
            }
            RoleDefinitionId::ResourceGroupScoped(resource_group_scoped_role_definition_id) => {
                resource_group_scoped_role_definition_id.name()
            }
            RoleDefinitionId::ResourceScoped(resource_scoped_role_definition_id) => {
                resource_scoped_role_definition_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for RoleDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleDefinitionId::Unscoped(UnscopedRoleDefinitionId {
            role_definition_name: name,
        })
    }
}
impl TryFromResourceGroupScoped for RoleDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleDefinitionId::ResourceGroupScoped(ResourceGroupScopedRoleDefinitionId {
            resource_group_id,
            name,
        })
    }
}
impl TryFromResourceScoped for RoleDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        RoleDefinitionId::ResourceScoped(ResourceScopedRoleDefinitionId { resource_id, name })
    }
}
impl TryFromSubscriptionScoped for RoleDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        RoleDefinitionId::SubscriptionScoped(SubscriptionScopedRoleDefinitionId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for RoleDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        RoleDefinitionId::ManagementGroupScoped(ManagementGroupScopedRoleDefinitionId {
            management_group_id,
            name,
        })
    }
}

// MARK: impl Scope
impl Scope for RoleDefinitionId {
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
        ScopeImplKind::RoleDefinition
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleDefinition(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for RoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for RoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}

// MARK: Serialize
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
        let id = RoleDefinitionId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}
