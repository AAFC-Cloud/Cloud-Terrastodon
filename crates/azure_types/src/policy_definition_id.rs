use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedPolicyDefinitionId;
use crate::prelude::PolicyDefinitionName;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedPolicyDefinitionId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedPolicyDefinitionId;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedPolicyDefinitionId;
use crate::prelude::UnscopedPolicyDefinitionId;
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

pub const POLICY_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicyDefinitionId {
    Unscoped(UnscopedPolicyDefinitionId),
    ManagementGroupScoped(ManagementGroupScopedPolicyDefinitionId),
    SubscriptionScoped(SubscriptionScopedPolicyDefinitionId),
    ResourceGroupScoped(ResourceGroupScopedPolicyDefinitionId),
    ResourceScoped(ResourceScopedPolicyDefinitionId),
}
impl PolicyDefinitionId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            PolicyDefinitionId::Unscoped(_) => None,
            PolicyDefinitionId::ManagementGroupScoped(_) => None,
            PolicyDefinitionId::SubscriptionScoped(subscription_scoped_role_assignment_id) => Some(
                subscription_scoped_role_assignment_id
                    .subscription_id()
                    .clone(),
            ),
            PolicyDefinitionId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(
                    resource_group_scoped_role_assignment_id
                        .subscription_id()
                        .clone(),
                )
            }
            PolicyDefinitionId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(resource_scoped_role_assignment_id.subscription_id().clone())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for PolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        match self {
            PolicyDefinitionId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            PolicyDefinitionId::ManagementGroupScoped(
                management_group_scoped_role_assignment_id,
            ) => management_group_scoped_role_assignment_id.name(),
            PolicyDefinitionId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                subscription_scoped_role_assignment_id.name()
            }
            PolicyDefinitionId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                resource_group_scoped_role_assignment_id.name()
            }
            PolicyDefinitionId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for PolicyDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        PolicyDefinitionId::Unscoped(UnscopedPolicyDefinitionId {
            name,
        })
    }
}
impl TryFromResourceGroupScoped for PolicyDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        PolicyDefinitionId::ResourceGroupScoped(ResourceGroupScopedPolicyDefinitionId {
            resource_group_id,
            name,
        })
    }
}
impl TryFromResourceScoped for PolicyDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        PolicyDefinitionId::ResourceScoped(ResourceScopedPolicyDefinitionId {
            resource_id,
            name,
        })
    }
}
impl TryFromSubscriptionScoped for PolicyDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        PolicyDefinitionId::SubscriptionScoped(SubscriptionScopedPolicyDefinitionId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for PolicyDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        PolicyDefinitionId::ManagementGroupScoped(ManagementGroupScopedPolicyDefinitionId {
            management_group_id,
            name,
        })
    }
}

// MARK: impl Scope
impl Scope for PolicyDefinitionId {
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
        ScopeImplKind::PolicyDefinition
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicyDefinition(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for PolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for PolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}

// MARK: Serialize
impl Serialize for PolicyDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyDefinitionId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;
    use eyre::bail;

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyDefinitionId::Unscoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyDefinitionId::ManagementGroupScoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyDefinitionId::SubscriptionScoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
            "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
            "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
        ] {
            let id: PolicyDefinitionId =
                serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
