use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::POLICY_SET_DEFINITION_ID_PREFIX;
use crate::prelude::PolicySetDefinitionId;
use crate::prelude::PolicySetDefinitionName;
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
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnscopedPolicySetDefinitionId {
    pub name: PolicySetDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedPolicySetDefinitionId {
    pub management_group_id: ManagementGroupId,
    pub name: PolicySetDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedPolicySetDefinitionId {
    pub subscription_id: SubscriptionId,
    pub name: PolicySetDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedPolicySetDefinitionId {
    pub resource_group_id: ResourceGroupId,
    pub name: PolicySetDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedPolicySetDefinitionId {
    pub resource_id: ResourceId,
    pub name: PolicySetDefinitionName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedPolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceGroupScopedPolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedPolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedPolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedPolicySetDefinitionId {
    type Name = PolicySetDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedPolicySetDefinitionId {}

impl ManagementGroupScoped for ManagementGroupScopedPolicySetDefinitionId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedPolicySetDefinitionId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedPolicySetDefinitionId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedPolicySetDefinitionId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedPolicySetDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self { name }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedPolicySetDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedPolicySetDefinitionId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedPolicySetDefinitionId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedPolicySetDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedPolicySetDefinitionId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedPolicySetDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedPolicySetDefinitionId {
    fn expanded_form(&self) -> String {
        format!("{POLICY_SET_DEFINITION_ID_PREFIX}{}", self.name)
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        UnscopedPolicySetDefinitionId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicySetDefinition(PolicySetDefinitionId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
}

impl Scope for ManagementGroupScopedPolicySetDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        ManagementGroupScopedPolicySetDefinitionId::try_from_expanded_management_group_scoped(
            expanded,
        )
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicySetDefinition(PolicySetDefinitionId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
}
impl Scope for SubscriptionScopedPolicySetDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        SubscriptionScopedPolicySetDefinitionId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicySetDefinition(PolicySetDefinitionId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
}

impl Scope for ResourceGroupScopedPolicySetDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        ResourceGroupScopedPolicySetDefinitionId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicySetDefinition(PolicySetDefinitionId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
}

impl Scope for ResourceScopedPolicySetDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        ResourceScopedPolicySetDefinitionId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicySetDefinition(PolicySetDefinitionId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedPolicySetDefinitionId {
    fn validate_name(name: &str) -> eyre::Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedPolicySetDefinitionId {
    fn validate_name(name: &str) -> eyre::Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedPolicySetDefinitionId {
    fn validate_name(name: &str) -> eyre::Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedPolicySetDefinitionId {
    fn validate_name(name: &str) -> eyre::Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedPolicySetDefinitionId {
    fn validate_name(name: &str) -> eyre::Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedPolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedPolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedPolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedPolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedPolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}

// MARK: FromStr

impl FromStr for UnscopedPolicySetDefinitionId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UnscopedPolicySetDefinitionId::try_from_expanded(s)
    }
}

impl FromStr for ManagementGroupScopedPolicySetDefinitionId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ManagementGroupScopedPolicySetDefinitionId::try_from_expanded(s)
    }
}

impl FromStr for SubscriptionScopedPolicySetDefinitionId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SubscriptionScopedPolicySetDefinitionId::try_from_expanded(s)
    }
}

impl FromStr for ResourceGroupScopedPolicySetDefinitionId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ResourceGroupScopedPolicySetDefinitionId::try_from_expanded(s)
    }
}

impl FromStr for ResourceScopedPolicySetDefinitionId {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ResourceScopedPolicySetDefinitionId::try_from_expanded(s)
    }
}
