use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScopedPolicySetDefinitionId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScopedPolicySetDefinitionId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScopedPolicySetDefinitionId;
use crate::prelude::PolicySetDefinitionName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SubscriptionScopedPolicySetDefinitionId;
use crate::prelude::UnscopedPolicySetDefinitionId;
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

pub const POLICY_SET_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policySetDefinitions/";
    
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicySetDefinitionId {
    Unscoped(UnscopedPolicySetDefinitionId),
    ManagementGroupScoped(ManagementGroupScopedPolicySetDefinitionId),
    SubscriptionScoped(SubscriptionScopedPolicySetDefinitionId),
    ResourceGroupScoped(ResourceGroupScopedPolicySetDefinitionId),
    ResourceScoped(ResourceScopedPolicySetDefinitionId),
}
impl PolicySetDefinitionId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            PolicySetDefinitionId::Unscoped(_) => None,
            PolicySetDefinitionId::ManagementGroupScoped(_) => None,
            PolicySetDefinitionId::SubscriptionScoped(subscription_scoped_role_assignment_id) => Some(
                subscription_scoped_role_assignment_id
                    .subscription_id()
                    .clone(),
            ),
            PolicySetDefinitionId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(
                    resource_group_scoped_role_assignment_id
                        .subscription_id()
                        .clone(),
                )
            }
            PolicySetDefinitionId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(resource_scoped_role_assignment_id.subscription_id().clone())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for PolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        match self {
            PolicySetDefinitionId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            PolicySetDefinitionId::ManagementGroupScoped(management_group_scoped_role_assignment_id) => {
                management_group_scoped_role_assignment_id.name()
            }
            PolicySetDefinitionId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                subscription_scoped_role_assignment_id.name()
            }
            PolicySetDefinitionId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                resource_group_scoped_role_assignment_id.name()
            }
            PolicySetDefinitionId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for PolicySetDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        PolicySetDefinitionId::Unscoped(UnscopedPolicySetDefinitionId {
            name,
        })
    }
}
impl TryFromResourceGroupScoped for PolicySetDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        PolicySetDefinitionId::ResourceGroupScoped(ResourceGroupScopedPolicySetDefinitionId {
            resource_group_id,
            name,
        })
    }
}
impl TryFromResourceScoped for PolicySetDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        PolicySetDefinitionId::ResourceScoped(ResourceScopedPolicySetDefinitionId {
            resource_id,
            name,
        })
    }
}
impl TryFromSubscriptionScoped for PolicySetDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        PolicySetDefinitionId::SubscriptionScoped(SubscriptionScopedPolicySetDefinitionId {
            subscription_id,
            name,
        })
    }
}
impl TryFromManagementGroupScoped for PolicySetDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        PolicySetDefinitionId::ManagementGroupScoped(ManagementGroupScopedPolicySetDefinitionId {
            management_group_id,
            name,
        })
    }
}


// MARK: impl Scope
impl Scope for PolicySetDefinitionId {
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
        ScopeImplKind::PolicySetDefinition
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicySetDefinition(self.clone())
    }
}

// MARK: NameValidatable

impl NameValidatable for PolicySetDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for PolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}



// MARK: Serialize
impl Serialize for PolicySetDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicySetDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicySetDefinitionId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}



#[cfg(test)]
mod tests {
    use crate::{prelude::ResourceGroupName, slug::Slug};

    use super::*;
    use cloud_terrastodon_azure_resource_types::prelude::ResourceType;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = PolicySetDefinitionId::Unscoped(UnscopedPolicySetDefinitionId {
            name: PolicySetDefinitionName::new("teehee"),
        });
        let id: PolicySetDefinitionId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/PolicySetDefinitions/{nil}
        let expanded = PolicySetDefinitionId::ResourceScoped(ResourceScopedPolicySetDefinitionId {
            resource_id: ResourceId::new(
                ResourceGroupId::new(
                    SubscriptionId::new(Uuid::new_v4()),
                    ResourceGroupName::try_new("MY-RG")?,
                ),
                ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
                "MY-VNET",
            ),
            name: PolicySetDefinitionName::new("Teehee"),
        });
        let id: PolicySetDefinitionId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
}
