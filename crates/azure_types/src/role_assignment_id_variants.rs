use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::RoleAssignmentId;
use crate::prelude::RoleAssignmentName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::Unscoped;
use crate::role_assignment_id::ROLE_ASSIGNMENT_ID_PREFIX;
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
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedRoleAssignmentId {
    pub role_assignment_name: RoleAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleAssignmentId {
    pub management_group_id: ManagementGroupId,
    pub name: RoleAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleAssignmentId {
    pub subscription_id: SubscriptionId,
    pub name: RoleAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleAssignmentId {
    pub resource_group_id: ResourceGroupId,
    pub name: RoleAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleAssignmentId {
    pub resource_id: ResourceId,
    pub name: RoleAssignmentName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedRoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.role_assignment_name
    }
}
impl HasSlug for ResourceGroupScopedRoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedRoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedRoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedRoleAssignmentId {
    type Name = RoleAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedRoleAssignmentId {}

impl ManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedRoleAssignmentId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedRoleAssignmentId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedRoleAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self {
            role_assignment_name: name,
        }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedRoleAssignmentId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleAssignmentId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedRoleAssignmentId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedRoleAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{ROLE_ASSIGNMENT_ID_PREFIX}{}",
            self.role_assignment_name.as_hyphenated()
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleAssignmentId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

impl Scope for ManagementGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleAssignmentId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl Scope for SubscriptionScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleAssignmentId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

impl Scope for ResourceGroupScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleAssignmentId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

impl Scope for ResourceScopedRoleAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleAssignmentId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleAssignment(RoleAssignmentId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedRoleAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedRoleAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_ASSIGNMENT_ID_PREFIX
    }
}

// MARK: FromStr

impl FromStr for UnscopedRoleAssignmentId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        UnscopedRoleAssignmentId::try_from_expanded(s)
    }
}

impl FromStr for ManagementGroupScopedRoleAssignmentId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ManagementGroupScopedRoleAssignmentId::try_from_expanded(s)
    }
}

impl FromStr for SubscriptionScopedRoleAssignmentId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        SubscriptionScopedRoleAssignmentId::try_from_expanded(s)
    }
}

impl FromStr for ResourceGroupScopedRoleAssignmentId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ResourceGroupScopedRoleAssignmentId::try_from_expanded(s)
    }
}

impl FromStr for ResourceScopedRoleAssignmentId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ResourceScopedRoleAssignmentId::try_from_expanded(s)
    }
}
