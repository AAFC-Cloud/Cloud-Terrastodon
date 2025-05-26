use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedPolicyAssignmentId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedPolicyAssignmentId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedPolicyAssignmentId;
use crate::prelude::PolicyAssignmentName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedPolicyAssignmentId;
use crate::prelude::UnscopedPolicyAssignmentId;
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


pub const POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyAssignments/";
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicyAssignmentId {
    Unscoped(UnscopedPolicyAssignmentId),
    ManagementGroupScoped(ManagementGroupScopedPolicyAssignmentId),
    SubscriptionScoped(SubscriptionScopedPolicyAssignmentId),
    ResourceGroupScoped(ResourceGroupScopedPolicyAssignmentId),
    ResourceScoped(ResourceScopedPolicyAssignmentId),
}
impl PolicyAssignmentId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            PolicyAssignmentId::Unscoped(_) => None,
            PolicyAssignmentId::ManagementGroupScoped(_) => None,
            PolicyAssignmentId::SubscriptionScoped(subscription_scoped_role_assignment_id) => Some(
                *subscription_scoped_role_assignment_id
                    .subscription_id(),
            ),
            PolicyAssignmentId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(
                    *resource_group_scoped_role_assignment_id
                        .subscription_id(),
                )
            }
            PolicyAssignmentId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(*resource_scoped_role_assignment_id.subscription_id())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for PolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        match self {
            PolicyAssignmentId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            PolicyAssignmentId::ManagementGroupScoped(management_group_scoped_role_assignment_id) => {
                management_group_scoped_role_assignment_id.name()
            }
            PolicyAssignmentId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                subscription_scoped_role_assignment_id.name()
            }
            PolicyAssignmentId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                resource_group_scoped_role_assignment_id.name()
            }
            PolicyAssignmentId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for PolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        PolicyAssignmentId::Unscoped(UnscopedPolicyAssignmentId {
            role_assignment_name: name,
        })
    }
}
impl TryFromResourceGroupScoped for PolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        PolicyAssignmentId::ResourceGroupScoped(ResourceGroupScopedPolicyAssignmentId {
            resource_group_id,
            name,
        })
    }
}
impl TryFromResourceScoped for PolicyAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        PolicyAssignmentId::ResourceScoped(ResourceScopedPolicyAssignmentId {
            resource_id,
            name,
        })
    }
}
impl TryFromSubscriptionScoped for PolicyAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        PolicyAssignmentId::SubscriptionScoped(SubscriptionScopedPolicyAssignmentId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for PolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        PolicyAssignmentId::ManagementGroupScoped(ManagementGroupScopedPolicyAssignmentId {
            management_group_id,
            name,
        })
    }
}


// MARK: impl Scope
impl Scope for PolicyAssignmentId {
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
        ScopeImplKind::PolicyAssignment
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicyAssignment(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for PolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for PolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}



// MARK: Serialize
impl Serialize for PolicyAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use eyre::{bail, Result};

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyAssignmentId::Unscoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "abc123");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::try_from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicyAssignmentId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyAssignmentId::ManagementGroupScoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "abc123");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyAssignments/GC Audit ISO 27001:20133";
        let id = PolicyAssignmentId::try_from_expanded_subscription_scoped(expanded)?;
        assert_eq!(id, PolicyAssignmentId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicyAssignmentId::SubscriptionScoped(_) => {}
            x => bail!("bad: {x:?}"),
        }
        assert_eq!(id.short_form(), "GC Audit ISO 27001:20133");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policyAssignments/abc123",
            "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyAssignments/abc123",
            "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyAssignments/GC Audit ISO 27001:20133",
        ] {
            let id: PolicyAssignmentId =
                serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
