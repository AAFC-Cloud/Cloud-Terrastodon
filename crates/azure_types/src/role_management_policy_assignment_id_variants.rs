use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::RoleManagementPolicyAssignmentId;
use crate::prelude::RoleManagementPolicyAssignmentName;
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
pub struct UnscopedRoleManagementPolicyAssignmentId {
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleManagementPolicyAssignmentId {
    pub management_group_id: ManagementGroupId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleManagementPolicyAssignmentId {
    pub subscription_id: SubscriptionId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleManagementPolicyAssignmentId {
    pub resource_group_id: ResourceGroupId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleManagementPolicyAssignmentId {
    pub resource_id: ResourceId,
    pub name: RoleManagementPolicyAssignmentName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedRoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedRoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedRoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedRoleManagementPolicyAssignmentId {}

impl ManagementGroupScoped for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedRoleManagementPolicyAssignmentId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedRoleManagementPolicyAssignmentId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedRoleManagementPolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self { name }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedRoleManagementPolicyAssignmentId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleManagementPolicyAssignmentId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedRoleManagementPolicyAssignmentId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedRoleManagementPolicyAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedRoleManagementPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!("{ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX}{}", self.name)
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleManagementPolicyAssignmentId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(RoleManagementPolicyAssignmentId::Unscoped(
            self.clone(),
        ))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
}

impl Scope for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleManagementPolicyAssignmentId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(
            RoleManagementPolicyAssignmentId::ManagementGroupScoped(self.clone()),
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
}
impl Scope for SubscriptionScopedRoleManagementPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleManagementPolicyAssignmentId::try_from_expanded_subscription_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(
            RoleManagementPolicyAssignmentId::SubscriptionScoped(self.clone()),
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
}

impl Scope for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleManagementPolicyAssignmentId::try_from_expanded_resource_group_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(
            RoleManagementPolicyAssignmentId::ResourceGroupScoped(self.clone()),
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
}

impl Scope for ResourceScopedRoleManagementPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleManagementPolicyAssignmentId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicyAssignment(RoleManagementPolicyAssignmentId::ResourceScoped(
            self.clone(),
        ))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicyAssignment
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedRoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedRoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedRoleManagementPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedRoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedRoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedRoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedRoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedRoleManagementPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ASSIGNMENT_ID_PREFIX
    }
}
