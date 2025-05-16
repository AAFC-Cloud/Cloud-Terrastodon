use crate::prelude::Fake;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::PrincipalId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::ResourceScoped;
use crate::prelude::RoleDefinitionId;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::Unscoped;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
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
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use uuid::Uuid;

pub const ROLE_ASSIGNMENT_ID_PREFIX: &str = "/providers/Microsoft.Authorization/roleAssignments/";

// TODO: learn how to make a derive or other macro to simplify this, cause this isn't going to scale well lol

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedRoleAssignmentId {
    expanded: String,
}
impl UnscopedRoleAssignmentId {
    pub fn new(uuid: &Uuid) -> Self {
        UnscopedRoleAssignmentId {
            expanded: format!("{ROLE_ASSIGNMENT_ID_PREFIX}{}", uuid.as_hyphenated()),
        }
    }
}
impl Unscoped for UnscopedRoleAssignmentId {}
impl NameValidatable for UnscopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for UnscopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromUnscoped for UnscopedRoleAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        UnscopedRoleAssignmentId {
            expanded: expanded.to_string(),
        }
    }
}
impl Fake for UnscopedRoleAssignmentId {
    fn fake() -> Self {
        UnscopedRoleAssignmentId::new(&Uuid::nil())
    }
}
impl Scope for UnscopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleAssignmentId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleAssignmentId {
    expanded: String,
}
impl NameValidatable for ManagementGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ManagementGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        ManagementGroupScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ManagementGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleAssignmentId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl ManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleAssignmentId {
    expanded: String,
}
impl NameValidatable for SubscriptionScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for SubscriptionScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        SubscriptionScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for SubscriptionScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleAssignmentId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for SubscriptionScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleAssignmentId {
    expanded: String,
    // subscription_id: &'a str,
    // resource_group_name: &'a str,
}

impl NameValidatable for ResourceGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ResourceGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        ResourceGroupScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ResourceGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleAssignmentId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for ResourceGroupScopedRoleAssignmentId {}
impl ResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleAssignmentId {
    expanded: String,
    // resource_id: &'a str
}

impl NameValidatable for ResourceScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for ResourceScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromResourceScoped for ResourceScopedRoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        ResourceScopedRoleAssignmentId {
            expanded: expanded.to_owned(),
        }
    }
}
impl Scope for ResourceScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        self.expanded.to_owned()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleAssignmentId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl SubscriptionScoped for ResourceScopedRoleAssignmentId {}
impl ResourceGroupScoped for ResourceScopedRoleAssignmentId {}
impl ResourceScoped for ResourceScopedRoleAssignmentId {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleAssignmentId {
    Unscoped(UnscopedRoleAssignmentId),
    ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId),
    SubscriptionScoped(SubscriptionScopedRoleAssignmentId),
    ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId),
    ResourceScoped(ResourceScopedRoleAssignmentId),
}
impl RoleAssignmentId {
    pub fn subscription_id(&self) -> Option<SubscriptionId> {
        match self {
            RoleAssignmentId::Unscoped(_) => None,
            RoleAssignmentId::ManagementGroupScoped(_) => None,
            RoleAssignmentId::SubscriptionScoped(subscription_scoped_role_assignment_id) => {
                Some(subscription_scoped_role_assignment_id.subscription_id())
            }
            RoleAssignmentId::ResourceGroupScoped(resource_group_scoped_role_assignment_id) => {
                Some(resource_group_scoped_role_assignment_id.subscription_id())
            }
            RoleAssignmentId::ResourceScoped(resource_scoped_role_assignment_id) => {
                Some(resource_scoped_role_assignment_id.subscription_id())
            }
        }
    }
}

impl NameValidatable for RoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for RoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl Fake for RoleAssignmentId {
    fn fake() -> Self {
        RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId::fake())
    }
}

impl TryFromUnscoped for RoleAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromResourceGroupScoped for RoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromResourceScoped for RoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromSubscriptionScoped for RoleAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::SubscriptionScoped(SubscriptionScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}
impl TryFromManagementGroupScoped for RoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleAssignmentId::ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId {
            expanded: expanded.to_string(),
        })
    }
}

impl Scope for RoleAssignmentId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        match self {
            Self::Unscoped(UnscopedRoleAssignmentId { expanded }) => expanded,
            Self::ResourceGroupScoped(ResourceGroupScopedRoleAssignmentId { expanded }) => expanded,
            Self::SubscriptionScoped(SubscriptionScopedRoleAssignmentId { expanded }) => expanded,
            Self::ManagementGroupScoped(ManagementGroupScopedRoleAssignmentId { expanded }) => {
                expanded
            }
            Self::ResourceScoped(ResourceScopedRoleAssignmentId { expanded }) => expanded,
        }.to_owned()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleAssignment(self.clone())
    }
}

impl Serialize for RoleAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#?}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RoleAssignment {
    pub id: RoleAssignmentId,
    pub scope: ResourceId,
    pub role_definition_id: RoleDefinitionId,
    pub principal_id: PrincipalId,
}

impl Fake for RoleAssignment {
    fn fake() -> Self {
        RoleAssignment {
            id: RoleAssignmentId::fake(),
            scope: ResourceId::new("SomeFakeResourceId"),
            role_definition_id: RoleDefinitionId::fake(),
            principal_id: PrincipalId::Unknown(Uuid::nil()),
        }
    }
}

impl HasScope for RoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &RoleAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl From<RoleAssignment> for HCLImportBlock {
    fn from(role_assignment: RoleAssignment) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            // Terraform doesn't like the case variation, https://github.com/hashicorp/terraform-provider-azurerm/issues/26907
            id: role_assignment
                .id
                .expanded_form()
                .replace("/RoleAssignments/", "/roleAssignments/"),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::RoleAssignment,
                name: role_assignment.id.short_form().sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = RoleAssignmentId::Unscoped(UnscopedRoleAssignmentId {
            expanded: format!(
                "{}{}",
                ROLE_ASSIGNMENT_ID_PREFIX, "55555555-5555-5555-5555-555555555555"
            ),
        });
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
    #[test]
    fn deserializes2() -> Result<()> {
        let nil = Uuid::nil();
        let expanded = RoleAssignmentId::ResourceScoped(ResourceScopedRoleAssignmentId {
            expanded: format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/roleAssignments/{nil}",
            ),
        });
        let id: RoleAssignmentId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }
}
