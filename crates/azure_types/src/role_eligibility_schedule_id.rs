use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedRoleEligibilityScheduleId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedRoleEligibilityScheduleId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedRoleEligibilityScheduleId;
use crate::prelude::RoleEligibilityScheduleName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedRoleEligibilityScheduleId;
use crate::prelude::UnscopedRoleEligibilityScheduleId;
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

pub const ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleEligibilitySchedules/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleEligibilityScheduleId {
    Unscoped(UnscopedRoleEligibilityScheduleId),
    ManagementGroupScoped(ManagementGroupScopedRoleEligibilityScheduleId),
    SubscriptionScoped(SubscriptionScopedRoleEligibilityScheduleId),
    ResourceGroupScoped(ResourceGroupScopedRoleEligibilityScheduleId),
    ResourceScoped(ResourceScopedRoleEligibilityScheduleId),
}
impl RoleEligibilityScheduleId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleEligibilityScheduleId::Unscoped(_) => None,
            RoleEligibilityScheduleId::ManagementGroupScoped(_) => None,
            RoleEligibilityScheduleId::SubscriptionScoped(
                subscription_scoped_role_assignment_id,
            ) => Some(*subscription_scoped_role_assignment_id.subscription_id()),
            RoleEligibilityScheduleId::ResourceGroupScoped(
                resource_group_scoped_role_assignment_id,
            ) => Some(*resource_group_scoped_role_assignment_id.subscription_id()),
            RoleEligibilityScheduleId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(*resource_scoped_role_assignment_id.subscription_id())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for RoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        match self {
            RoleEligibilityScheduleId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            RoleEligibilityScheduleId::ManagementGroupScoped(
                management_group_scoped_role_assignment_id,
            ) => management_group_scoped_role_assignment_id.name(),
            RoleEligibilityScheduleId::SubscriptionScoped(
                subscription_scoped_role_assignment_id,
            ) => subscription_scoped_role_assignment_id.name(),
            RoleEligibilityScheduleId::ResourceGroupScoped(
                resource_group_scoped_role_assignment_id,
            ) => resource_group_scoped_role_assignment_id.name(),
            RoleEligibilityScheduleId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for RoleEligibilityScheduleId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleEligibilityScheduleId::Unscoped(UnscopedRoleEligibilityScheduleId { name })
    }
}
impl TryFromResourceGroupScoped for RoleEligibilityScheduleId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleEligibilityScheduleId::ResourceGroupScoped(
            ResourceGroupScopedRoleEligibilityScheduleId {
                resource_group_id,
                name,
            },
        )
    }
}
impl TryFromResourceScoped for RoleEligibilityScheduleId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        RoleEligibilityScheduleId::ResourceScoped(ResourceScopedRoleEligibilityScheduleId {
            resource_id,
            name,
        })
    }
}
impl TryFromSubscriptionScoped for RoleEligibilityScheduleId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        RoleEligibilityScheduleId::SubscriptionScoped(SubscriptionScopedRoleEligibilityScheduleId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for RoleEligibilityScheduleId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        RoleEligibilityScheduleId::ManagementGroupScoped(
            ManagementGroupScopedRoleEligibilityScheduleId {
                management_group_id,
                name,
            },
        )
    }
}

// MARK: impl Scope
impl Scope for RoleEligibilityScheduleId {
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
        ScopeImplKind::RoleEligibilitySchedule
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for RoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for RoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}

// MARK: Serialize
impl Serialize for RoleEligibilityScheduleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleEligibilityScheduleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleEligibilityScheduleId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::slug::Slug;
    use cloud_terrastodon_azure_resource_types::ResourceType;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = RoleEligibilityScheduleId::Unscoped(UnscopedRoleEligibilityScheduleId {
            name: RoleEligibilityScheduleName::new(Uuid::nil()),
        });
        let id: RoleEligibilityScheduleId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/RoleEligibilitySchedules/{nil}
        let expanded =
            RoleEligibilityScheduleId::ResourceScoped(ResourceScopedRoleEligibilityScheduleId {
                resource_id: ResourceId::new(
                    ResourceGroupId::new(
                        SubscriptionId::new(Uuid::new_v4()),
                        ResourceGroupName::try_new("MY-RG")?,
                    ),
                    ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
                    "MY-VNET",
                ),
                name: RoleEligibilityScheduleName::new(Uuid::new_v4()),
            });
        let id: RoleEligibilityScheduleId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
}
