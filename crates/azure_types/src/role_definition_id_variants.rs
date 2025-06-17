use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ROLE_DEFINITION_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleDefinitionName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::Unscoped;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::ResourceScoped;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromResourceScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use crate::slug::HasSlug;
use eyre::Result;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedRoleDefinitionId {
    pub role_definition_name: RoleDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleDefinitionId {
    pub management_group_id: ManagementGroupId,
    pub name: RoleDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleDefinitionId {
    pub subscription_id: SubscriptionId,
    pub name: RoleDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleDefinitionId {
    pub resource_group_id: ResourceGroupId,
    pub name: RoleDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleDefinitionId {
    pub resource_id: ResourceId,
    pub name: RoleDefinitionName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedRoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.role_definition_name
    }
}
impl HasSlug for ResourceGroupScopedRoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedRoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedRoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedRoleDefinitionId {
    type Name = RoleDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedRoleDefinitionId {}

impl ManagementGroupScoped for ManagementGroupScopedRoleDefinitionId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedRoleDefinitionId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedRoleDefinitionId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedRoleDefinitionId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedRoleDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self {
            role_definition_name: name,
        }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedRoleDefinitionId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        Self {
            subscription_id,
            name,
        }
    }
}
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedRoleDefinitionId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedRoleDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedRoleDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{ROLE_DEFINITION_ID_PREFIX}{}",
            self.role_definition_name.as_hyphenated()
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleDefinitionId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleDefinition(RoleDefinitionId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
}

impl Scope for ManagementGroupScopedRoleDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleDefinitionId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleDefinition(RoleDefinitionId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
}
impl Scope for SubscriptionScopedRoleDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleDefinitionId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleDefinition(RoleDefinitionId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
}

impl Scope for ResourceGroupScopedRoleDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleDefinitionId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleDefinition(RoleDefinitionId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
}

impl Scope for ResourceScopedRoleDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleDefinitionId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleDefinition(RoleDefinitionId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleDefinition
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedRoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedRoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedRoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedRoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedRoleDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedRoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedRoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedRoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedRoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedRoleDefinitionId {
    fn get_prefix() -> &'static str {
        ROLE_DEFINITION_ID_PREFIX
    }
}
