use crate::ManagementGroupId;
use crate::ManagementGroupScopedRoleAssignmentId;
use crate::PortalScopedRoleAssignmentId;
use crate::ResourceGroupId;
use crate::ResourceGroupScopedRoleAssignmentId;
use crate::ResourceId;
use crate::ResourceScopedRoleAssignmentId;
use crate::RoleAssignmentName;
use crate::ServiceGroupId;
use crate::ServiceGroupScopedRoleAssignmentId;
use crate::SubscriptionId;
use crate::SubscriptionScoped;
use crate::SubscriptionScopedRoleAssignmentId;
use crate::UnscopedRoleAssignmentId;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromPortalScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromResourceScoped;
use crate::scopes::TryFromServiceGroupScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use crate::scopes::try_from_expanded_hierarchy_scoped_with_portal;
use crate::slug::HasSlug;
use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use uuid::Uuid;

pub const ROLE_ASSIGNMENT_ID_PREFIX: &str = "/providers/Microsoft.Authorization/roleAssignments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum RoleAssignmentId {
    Unscoped(UnscopedRoleAssignmentId),
    PortalScoped(PortalScopedRoleAssignmentId),
    ServiceGroupScoped(ServiceGroupScopedRoleAssignmentId),
    ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId),
    SubscriptionScoped(SubscriptionScopedRoleAssignmentId),
    ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId),
    ResourceScoped(ResourceScopedRoleAssignmentId),
}
crate::impl_facet_string_proxy!(RoleAssignmentId, value => value.expanded_form());
impl<'a> Arbitrary<'a> for RoleAssignmentId {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=2)? {
            0 => Self::Unscoped(UnscopedRoleAssignmentId {
                role_assignment_name: RoleAssignmentName::arbitrary(u)?,
            }),
            1 => Self::PortalScoped(PortalScopedRoleAssignmentId {
                role_assignment_name: RoleAssignmentName::arbitrary(u)?,
            }),
            _ => Self::SubscriptionScoped(SubscriptionScopedRoleAssignmentId {
                subscription_id: SubscriptionId::arbitrary(u)?,
                role_assignment_name: RoleAssignmentName::arbitrary(u)?,
            }),
        })
    }
}
impl RoleAssignmentId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleAssignmentId::Unscoped(_) => None,
            RoleAssignmentId::PortalScoped(_) => None,
            RoleAssignmentId::ServiceGroupScoped(_) => None,
            RoleAssignmentId::ManagementGroupScoped(_) => None,
            RoleAssignmentId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                Some(*subscription_scoped_role_assignment_id.subscription_id())
            }
            RoleAssignmentId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(*resource_group_scoped_role_assignment_id.subscription_id())
            }
            RoleAssignmentId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(*resource_scoped_role_assignment_id.subscription_id())
            }
        }
    }
}

// MARK: HasSlug
impl HasSlug for RoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        match self {
            RoleAssignmentId::Unscoped(unscoped_role_assignment_id) => {
                unscoped_role_assignment_id.name()
            }
            RoleAssignmentId::PortalScoped(portal_scoped_role_assignment_id) => {
                portal_scoped_role_assignment_id.name()
            }
            RoleAssignmentId::ServiceGroupScoped(service_group_scoped_role_assignment_id) => {
                service_group_scoped_role_assignment_id.name()
            }
            RoleAssignmentId::ManagementGroupScoped(management_group_scoped_role_assignment_id) => {
                management_group_scoped_role_assignment_id.name()
            }
            RoleAssignmentId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                subscription_scoped_role_assignment_id.name()
            }
            RoleAssignmentId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                resource_group_scoped_role_assignment_id.name()
            }
            RoleAssignmentId::ResourceScoped(resource_scoped_role_assignment_id) => {
                resource_scoped_role_assignment_id.name()
            }
        }
    }
}

// MARK: TryFrom

impl TryFromUnscoped for RoleAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            role_assignment_name: name,
        })
    }
}
impl TryFromPortalScoped for RoleAssignmentId {
    unsafe fn new_portal_scoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        RoleAssignmentId::PortalScoped(PortalScopedRoleAssignmentId {
            role_assignment_name: name,
        })
    }
}
impl TryFromServiceGroupScoped for RoleAssignmentId {
    unsafe fn new_service_group_scoped_unchecked(
        _expanded: &str,
        service_group_id: ServiceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleAssignmentId::ServiceGroupScoped(ServiceGroupScopedRoleAssignmentId {
            service_group_id,
            role_assignment_name: name,
        })
    }
}
impl TryFromResourceGroupScoped for RoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        RoleAssignmentId::ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId {
            resource_group_id,
            role_assignment_name: name,
        })
    }
}
impl TryFromResourceScoped for RoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            resource_id,
            role_assignment_name: name,
        })
    }
}
impl TryFromSubscriptionScoped for RoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        RoleAssignmentId::SubscriptionScoped(SubscriptionScopedRoleAssignmentId {
            subscription_id,
            role_assignment_name: name,
        })
    }
}
impl TryFromManagementGroupScoped for RoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        RoleAssignmentId::ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId {
            management_group_id,
            role_assignment_name: name,
        })
    }
}

// MARK: impl Scope
impl Scope for RoleAssignmentId {
    type Err = <Self as std::str::FromStr>::Err;
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped_with_portal(expanded)
    }

    fn expanded_form(&self) -> String {
        match self {
            Self::Unscoped(x) => x.expanded_form(),
            Self::PortalScoped(x) => x.expanded_form(),
            Self::ServiceGroupScoped(x) => x.expanded_form(),
            Self::ResourceGroupScoped(x) => x.expanded_form(),
            Self::SubscriptionScoped(x) => x.expanded_form(),
            Self::ManagementGroupScoped(x) => x.expanded_form(),
            Self::ResourceScoped(x) => x.expanded_form(),
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleAssignment(self.clone())
    }
}

impl FromStr for RoleAssignmentId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        RoleAssignmentId::try_from_expanded(s)
    }
}

// MARK: NameValidatable

impl NameValidatable for RoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix

impl HasPrefix for RoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentId);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentId);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupName;
    use crate::ServiceGroupName;
    use crate::slug::Slug;
    use cloud_terrastodon_azure_resource_types::ResourceType;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            role_assignment_name: RoleAssignmentName::new(Uuid::nil()),
        });
        let json = facet_json::to_string(&expanded)?;
        assert_eq!(json, facet_json::to_string(&expanded.expanded_form())?);
        let id: RoleAssignmentId = facet_json::from_str(json.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/roleAssignments/{nil}
        let expanded = RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            resource_id: ResourceId::new(
                ResourceGroupId::new(
                    SubscriptionId::new(Uuid::new_v4()),
                    ResourceGroupName::try_new("MY-RG")?,
                ),
                ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
                "MY-VNET",
            ),
            role_assignment_name: RoleAssignmentName::new(Uuid::new_v4()),
        });
        let json = facet_json::to_string(&expanded)?;
        assert_eq!(json, facet_json::to_string(&expanded.expanded_form())?);
        let id: RoleAssignmentId = facet_json::from_str(json.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn deserializes_service_group() -> Result<()> {
        let service_group_id =
            ServiceGroupId::from_name(ServiceGroupName::try_new("MyServiceGroup")?);
        let expanded = RoleAssignmentId::ServiceGroupScoped(ServiceGroupScopedRoleAssignmentId {
            service_group_id: service_group_id.clone(),
            role_assignment_name: RoleAssignmentName::new(Uuid::new_v4()),
        });
        let json = facet_json::to_string(&expanded)?;
        assert_eq!(json, facet_json::to_string(&expanded.expanded_form())?);
        let id: RoleAssignmentId = facet_json::from_str(json.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn hashmap_keys_use_string_proxy_serialization() -> Result<()> {
        use std::collections::HashMap;

        let first = RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            role_assignment_name: RoleAssignmentName::new(uuid::uuid!(
                "00000000-0000-0000-0000-000000000001"
            )),
        });
        let second = RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            role_assignment_name: RoleAssignmentName::new(uuid::uuid!(
                "00000000-0000-0000-0000-000000000002"
            )),
        });
        assert_ne!(first.expanded_form(), second.expanded_form());
        assert_eq!(
            facet_json::to_string(&first)?,
            facet_json::to_string(&first.expanded_form())?
        );

        let mut lookup = HashMap::new();
        lookup.insert(first.clone(), "first");
        lookup.insert(second.clone(), "second");
        assert_eq!(lookup.len(), 2);

        let standalone_json = facet_json::to_string(&first)?;
        let json = facet_json::to_string(&lookup)?;
        let pretty_json = facet_json::to_string_pretty(&lookup)?;
        println!("standalone RoleAssignmentId JSON:\n{standalone_json}");
        println!("HashMap<RoleAssignmentId, &str> JSON:\n{pretty_json}");

        let first_key_json = facet_json::to_string(&first.expanded_form())?;
        let second_key_json = facet_json::to_string(&second.expanded_form())?;
        assert!(
            json.contains(&first_key_json),
            "map-key JSON should use the first expanded id as an object key: {json}"
        );
        assert!(
            json.contains(&second_key_json),
            "map-key JSON should use the second expanded id as an object key: {json}"
        );
        assert_eq!(
            json.matches("\"⟨RoleAssignmentId⟩\"").count(),
            0,
            "map-key JSON should not use the facet placeholder as a map key: {json}"
        );

        Ok(())
    }

    #[test]
    fn parses_service_group_scope() -> Result<()> {
        let service_group_id =
            ServiceGroupId::from_name(ServiceGroupName::try_new("MyServiceGroup")?);
        let name = RoleAssignmentName::new(Uuid::new_v4());
        let expanded = format!(
            "{}{}{}",
            service_group_id.expanded_form(),
            ROLE_ASSIGNMENT_ID_PREFIX,
            name
        );
        let id = RoleAssignmentId::try_from_expanded(&expanded)?;
        match id {
            RoleAssignmentId::ServiceGroupScoped(parsed) => {
                assert_eq!(parsed.service_group_id, service_group_id);
            }
            other => panic!("expected service group scoped, got {other:?}"),
        }
        Ok(())
    }
}
