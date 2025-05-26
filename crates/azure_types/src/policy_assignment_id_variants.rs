use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::POLICY_ASSIGNMENT_ID_PREFIX;
use crate::prelude::PolicyAssignmentId;
use crate::prelude::PolicyAssignmentName;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
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
pub struct UnscopedPolicyAssignmentId {
    pub role_assignment_name: PolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedPolicyAssignmentId {
    pub management_group_id: ManagementGroupId,
    pub name: PolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedPolicyAssignmentId {
    pub subscription_id: SubscriptionId,
    pub name: PolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedPolicyAssignmentId {
    pub resource_group_id: ResourceGroupId,
    pub name: PolicyAssignmentName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedPolicyAssignmentId {
    pub resource_id: ResourceId,
    pub name: PolicyAssignmentName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedPolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.role_assignment_name
    }
}
impl HasSlug for ResourceGroupScopedPolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedPolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedPolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedPolicyAssignmentId {
    type Name = PolicyAssignmentName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedPolicyAssignmentId {}

impl ManagementGroupScoped for ManagementGroupScopedPolicyAssignmentId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedPolicyAssignmentId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedPolicyAssignmentId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedPolicyAssignmentId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedPolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self {
            role_assignment_name: name,
        }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedPolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedPolicyAssignmentId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedPolicyAssignmentId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedPolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedPolicyAssignmentId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedPolicyAssignmentId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!("{POLICY_ASSIGNMENT_ID_PREFIX}{}", self.role_assignment_name)
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedPolicyAssignmentId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyAssignment(PolicyAssignmentId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyAssignment
    }
}

impl Scope for ManagementGroupScopedPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedPolicyAssignmentId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyAssignment(PolicyAssignmentId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyAssignment
    }
}
impl Scope for SubscriptionScopedPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedPolicyAssignmentId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyAssignment(PolicyAssignmentId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyAssignment
    }
}

impl Scope for ResourceGroupScopedPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedPolicyAssignmentId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyAssignment(PolicyAssignmentId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyAssignment
    }
}

impl Scope for ResourceScopedPolicyAssignmentId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedPolicyAssignmentId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyAssignment(PolicyAssignmentId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyAssignment
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedPolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedPolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
