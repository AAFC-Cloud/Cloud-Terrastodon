use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ROLE_MANAGEMENT_POLICY_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::RoleManagementPolicyAssignmentName;
use crate::prelude::RoleManagementPolicyId;
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
pub struct UnscopedRoleManagementPolicyId {
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleManagementPolicyId {
    pub management_group_id: ManagementGroupId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleManagementPolicyId {
    pub subscription_id: SubscriptionId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleManagementPolicyId {
    pub resource_group_id: ResourceGroupId,
    pub name: RoleManagementPolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleManagementPolicyId {
    pub resource_id: ResourceId,
    pub name: RoleManagementPolicyAssignmentName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedRoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceGroupScopedRoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedRoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedRoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedRoleManagementPolicyId {
    type Name = RoleManagementPolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedRoleManagementPolicyId {}

impl ManagementGroupScoped for ManagementGroupScopedRoleManagementPolicyId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedRoleManagementPolicyId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedRoleManagementPolicyId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedRoleManagementPolicyId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedRoleManagementPolicyId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self {
            name,
        }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleManagementPolicyId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedRoleManagementPolicyId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleManagementPolicyId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleManagementPolicyId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedRoleManagementPolicyId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedRoleManagementPolicyId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_id,
            name,
        }
    }
}

// MARK: impl Scope

impl Scope for UnscopedRoleManagementPolicyId {
    fn expanded_form(&self) -> String {
        format!(
            "{ROLE_MANAGEMENT_POLICY_ID_PREFIX}{}",
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleManagementPolicyId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicy(RoleManagementPolicyId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

impl Scope for ManagementGroupScopedRoleManagementPolicyId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleManagementPolicyId::try_from_expanded_management_group_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicy(RoleManagementPolicyId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}
impl Scope for SubscriptionScopedRoleManagementPolicyId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleManagementPolicyId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicy(RoleManagementPolicyId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleAssignment
    }
}

impl Scope for ResourceGroupScopedRoleManagementPolicyId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleManagementPolicyId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicy(RoleManagementPolicyId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicy
    }
}

impl Scope for ResourceScopedRoleManagementPolicyId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleManagementPolicyId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleManagementPolicy(RoleManagementPolicyId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleManagementPolicy
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedRoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedRoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedRoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedRoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedRoleManagementPolicyId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedRoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedRoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedRoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedRoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedRoleManagementPolicyId {
    fn get_prefix() -> &'static str {
        ROLE_MANAGEMENT_POLICY_ID_PREFIX
    }
}
