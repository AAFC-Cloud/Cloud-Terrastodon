use crate::prelude::ManagementGroupId;
use crate::prelude::ManagementGroupScoped;
use crate::prelude::POLICY_DEFINITION_ID_PREFIX;
use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicyDefinitionName;
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
pub struct UnscopedPolicyDefinitionId {
    pub name: PolicyDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ManagementGroupScopedPolicyDefinitionId {
    pub management_group_id: ManagementGroupId,
    pub name: PolicyDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionScopedPolicyDefinitionId {
    pub subscription_id: SubscriptionId,
    pub name: PolicyDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupScopedPolicyDefinitionId {
    pub resource_group_id: ResourceGroupId,
    pub name: PolicyDefinitionName,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceScopedPolicyDefinitionId {
    pub resource_id: ResourceId,
    pub name: PolicyDefinitionName,
}

// MARK: impl HasSlug

impl HasSlug for UnscopedPolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceGroupScopedPolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ResourceScopedPolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for SubscriptionScopedPolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}
impl HasSlug for ManagementGroupScopedPolicyDefinitionId {
    type Name = PolicyDefinitionName;

    fn name(&self) -> &Self::Name {
        &self.name
    }
}

impl Unscoped for UnscopedPolicyDefinitionId {}

impl ManagementGroupScoped for ManagementGroupScopedPolicyDefinitionId {
    fn management_group_id(&self) -> &ManagementGroupId {
        &self.management_group_id
    }
}
impl ResourceGroupScoped for ResourceGroupScopedPolicyDefinitionId {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl ResourceScoped for ResourceScopedPolicyDefinitionId {
    fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
}

impl SubscriptionScoped for SubscriptionScopedPolicyDefinitionId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

// MARK: TryFrom
impl TryFromUnscoped for UnscopedPolicyDefinitionId {
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self {
        Self { name }
    }
}
impl TryFromManagementGroupScoped for ManagementGroupScopedPolicyDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self {
        ManagementGroupScopedPolicyDefinitionId {
            management_group_id,
            name,
        }
    }
}
impl TryFromSubscriptionScoped for SubscriptionScopedPolicyDefinitionId {
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
impl TryFromResourceGroupScoped for ResourceGroupScopedPolicyDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: crate::prelude::ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ResourceGroupScopedPolicyDefinitionId {
            resource_group_id,
            name,
        }
    }
}
impl TryFromResourceScoped for ResourceScopedPolicyDefinitionId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self {
        Self { resource_id, name }
    }
}

// MARK: impl Scope

impl Scope for UnscopedPolicyDefinitionId {
    fn expanded_form(&self) -> String {
        format!("{POLICY_DEFINITION_ID_PREFIX}{}", self.name)
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        UnscopedPolicyDefinitionId::try_from_expanded_unscoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyDefinition(PolicyDefinitionId::Unscoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
}

impl Scope for ManagementGroupScopedPolicyDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.management_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ManagementGroupScopedPolicyDefinitionId::try_from_expanded_management_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyDefinition(PolicyDefinitionId::ManagementGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
}
impl Scope for SubscriptionScopedPolicyDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        SubscriptionScopedPolicyDefinitionId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyDefinition(PolicyDefinitionId::SubscriptionScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
}

impl Scope for ResourceGroupScopedPolicyDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupScopedPolicyDefinitionId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyDefinition(PolicyDefinitionId::ResourceGroupScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
}

impl Scope for ResourceScopedPolicyDefinitionId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_id.expanded_form(),
            Self::get_prefix(),
            self.name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceScopedPolicyDefinitionId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::PolicyDefinition(PolicyDefinitionId::ResourceScoped(self.clone()))
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
}

// MARK: FromStr

impl FromStr for UnscopedPolicyDefinitionId {
    type Err = eyre::Error;
    fn from_str(expanded: &str) -> Result<Self> {
        UnscopedPolicyDefinitionId::try_from_expanded_unscoped(expanded)
    }
}

impl FromStr for ManagementGroupScopedPolicyDefinitionId {
    type Err = eyre::Error;
    fn from_str(expanded: &str) -> Result<Self> {
        ManagementGroupScopedPolicyDefinitionId::try_from_expanded_management_group_scoped(expanded)
    }
}
impl FromStr for SubscriptionScopedPolicyDefinitionId {
    type Err = eyre::Error;
    fn from_str(expanded: &str) -> Result<Self> {
        SubscriptionScopedPolicyDefinitionId::try_from_expanded_subscription_scoped(expanded)
    }
}
impl FromStr for ResourceGroupScopedPolicyDefinitionId {
    type Err = eyre::Error;
    fn from_str(expanded: &str) -> Result<Self> {
        ResourceGroupScopedPolicyDefinitionId::try_from_expanded_resource_group_scoped(expanded)
    }
}
impl FromStr for ResourceScopedPolicyDefinitionId {
    type Err = eyre::Error;
    fn from_str(expanded: &str) -> Result<Self> {
        ResourceScopedPolicyDefinitionId::try_from_expanded_resource_scoped(expanded)
    }
}

// MARK: NameValidatable

impl NameValidatable for UnscopedPolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

impl NameValidatable for ManagementGroupScopedPolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for SubscriptionScopedPolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceGroupScopedPolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl NameValidatable for ResourceScopedPolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}

// MARK: HasPrefix
impl HasPrefix for UnscopedPolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ManagementGroupScopedPolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for SubscriptionScopedPolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceGroupScopedPolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
impl HasPrefix for ResourceScopedPolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
