use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupScoped;
use crate::prelude::ResourceId;
use crate::prelude::RoleEligibilityScheduleId;
use crate::prelude::RoleEligibilityScheduleName;
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
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedRoleEligibilityScheduleId {
    pub name: RoleEligibilityScheduleName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedRoleEligibilityScheduleId {
    pub management_group_id: ManagementGroupId,
    pub name: RoleEligibilityScheduleName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedRoleEligibilityScheduleId {
    pub subscription_id: SubscriptionId,
    pub name: RoleEligibilityScheduleName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedRoleEligibilityScheduleId {
    pub resource_group_id: ResourceGroupId,
    pub name: RoleEligibilityScheduleName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedRoleEligibilityScheduleId {
    pub resource_id: ResourceId,
    pub name: RoleEligibilityScheduleName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedRoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceGroupScopedRoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedRoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedRoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedRoleEligibilityScheduleId {
    type Name = RoleEligibilityScheduleName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedRoleEligibilityScheduleId {}

impl ManagementGroupScoped for ManagementGroupScopedRoleEligibilityScheduleId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedRoleEligibilityScheduleId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedRoleEligibilityScheduleId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedRoleEligibilityScheduleId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedRoleEligibilityScheduleId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self { name }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedRoleEligibilityScheduleId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedRoleEligibilityScheduleId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedRoleEligibilityScheduleId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedRoleEligibilityScheduleId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedRoleEligibilityScheduleId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedRoleEligibilityScheduleId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedRoleEligibilityScheduleId {
    fn expanded_form(&self) -> String {
        format!(
            "{ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX}{}",
            self.name.as_hyphenated()
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedRoleEligibilityScheduleId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(RoleEligibilityScheduleId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
}

impl Scope for ManagementGroupScopedRoleEligibilityScheduleId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedRoleEligibilityScheduleId::try_from_expanded_management_group_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(RoleEligibilityScheduleId::ManagementGroupScoped(
            self.clone(),
        ))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
}
impl Scope for SubscriptionScopedRoleEligibilityScheduleId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedRoleEligibilityScheduleId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(RoleEligibilityScheduleId::SubscriptionScoped(
            self.clone(),
        ))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
}

impl Scope for ResourceGroupScopedRoleEligibilityScheduleId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedRoleEligibilityScheduleId::try_from_expanded_resource_group_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(RoleEligibilityScheduleId::ResourceGroupScoped(
            self.clone(),
        ))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
}

impl Scope for ResourceScopedRoleEligibilityScheduleId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedRoleEligibilityScheduleId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(RoleEligibilityScheduleId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedRoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedRoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedRoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedRoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedRoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedRoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedRoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedRoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedRoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedRoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}

// MARK: FromStr

impl FromStr for UnscopedRoleEligibilityScheduleId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        UnscopedRoleEligibilityScheduleId::try_from_expanded(s)
    }
}

impl FromStr for ManagementGroupScopedRoleEligibilityScheduleId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ManagementGroupScopedRoleEligibilityScheduleId::try_from_expanded(s)
    }
}

impl FromStr for SubscriptionScopedRoleEligibilityScheduleId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        SubscriptionScopedRoleEligibilityScheduleId::try_from_expanded(s)
    }
}

impl FromStr for ResourceGroupScopedRoleEligibilityScheduleId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ResourceGroupScopedRoleEligibilityScheduleId::try_from_expanded(s)
    }
}

impl FromStr for ResourceScopedRoleEligibilityScheduleId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self> {
        ResourceScopedRoleEligibilityScheduleId::try_from_expanded(s)
    }
}
