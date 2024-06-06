use crate::management_groups::ManagementGroupId;
use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;
use crate::policy_assignments::PolicyAssignmentId;
use crate::policy_definitions::PolicyDefinitionId;
use crate::policy_set_definitions::PolicySetDefinitionId;
use crate::prelude::ResourceGroupId;
use crate::prelude::RoleAssignmentId;
use crate::subscriptions::SUBSCRIPTION_ID_PREFIX;
use anyhow::bail;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use std::str::FromStr;

pub trait Scope: Sized {
    fn expanded_form(&self) -> &str;
    fn short_form(&self) -> &str;
    fn try_from_expanded(expanded: &str) -> Result<Self>;
}
pub trait NameValidatable {
    fn validate_name(name: &str) -> Result<()>;
}
pub trait HasPrefix {
    fn get_prefix() -> Option<&'static str>;
}
pub trait TryFromUnscoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_unscoped(expanded_unscoped: &str) -> Result<Self> {
        // Get name without prefix
        let name = match Self::get_prefix() {
            None => expanded_unscoped,
            Some(prefix) => match expanded_unscoped.strip_prefix(prefix) {
                None => {
                    return Err(ScopeError::Malformed).context(format!(
                        "Unscoped expanded form {expanded_unscoped:?} must begin with prefix {prefix:?}"
                    ))
                }
                Some(name) => name,
            },
        };
        Self::validate_name(name)?;

        unsafe { Ok(Self::new_unscoped_unchecked(expanded_unscoped)) }
    }
    /// # Safety
    /// 
    /// The try_from methods should be used instead
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self;
}

pub trait TryFromSubscriptionScoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_subscription_scoped(expanded: &str) -> Result<Self> {
        // Remove subscription prefix
        let Some(remaining) = expanded.strip_prefix(SUBSCRIPTION_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
                .context(format!("Subscription scoped expanded form {expanded:?} must begin with {SUBSCRIPTION_ID_PREFIX:?}"));
        };

        // Remove subscription id
        let Some((_sub_name, remaining)) = remaining.split_once('/') else {
            return Err(ScopeError::Malformed).context(format!("Subscription scoped expanded form {expanded:?} must contain a slash after the prefix {SUBSCRIPTION_ID_PREFIX:?}"));
        };
        let expanded_unscoped = &expanded[expanded.len() - remaining.len() - 1..]; // Keep leading slash

        // Get name without prefix
        let name = match Self::get_prefix() {
            None => expanded_unscoped,
            Some(prefix) => match expanded_unscoped.strip_prefix(prefix) {
                None => {
                    return Err(ScopeError::Malformed).context(format!("Stripped expanded form {expanded_unscoped:?} (from {expanded:?}) must begin with {prefix:?}"
                    ))
                }
                Some(name) => name,
            },
        };

        Self::validate_name(name)?;

        unsafe { Ok(Self::new_subscription_scoped_unchecked(expanded)) }
    }
    
    /// # Safety
    /// 
    /// The try_from methods should be used instead
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self;
}
pub trait TryFromManagementGroupScoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_management_group_scoped(expanded: &str) -> Result<Self> {
        // Remove management group prefix
        let Some(remaining) = expanded.strip_prefix(MANAGEMENT_GROUP_ID_PREFIX) else {
            return Err(ScopeError::Malformed)
                .context(format!("Management group scoped expanded form {expanded:?} must begin with {MANAGEMENT_GROUP_ID_PREFIX:?}"));
        };

        // Remove management group name
        let Some((_management_group_name, remaining)) = remaining.split_once('/') else {
            return Err(ScopeError::Malformed).context(format!("Management group scoped expanded form {expanded:?} must contain a slash after the prefix {MANAGEMENT_GROUP_ID_PREFIX:?}"));
        };
        let expanded_unscoped = &expanded[expanded.len() - remaining.len() - 1..]; // Keep leading slash

        // Get name without prefix
        let name = match Self::get_prefix() {
            None => expanded_unscoped,
            Some(prefix) => match expanded_unscoped.strip_prefix(prefix) {
                None => {
                    return Err(ScopeError::Malformed).context(format!("Stripped expanded form {expanded_unscoped:?} (from {expanded:?}) must begin with {prefix:?}"
                    ))
                }
                Some(name) => name,
            },
        };

        Self::validate_name(name)?;

        unsafe { Ok(Self::new_management_group_scoped_unchecked(expanded)) }
    }
    /// # Safety
    /// 
    /// The try_from methods should be used instead
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self;
}

pub fn try_from_expanded_hierarchy_scoped<T>(expanded: &str) -> Result<T>
where
    T: TryFromUnscoped + TryFromManagementGroupScoped + TryFromSubscriptionScoped,
{
    match T::try_from_expanded_management_group_scoped(expanded) {
        Ok(x) => Ok(x),
        Err(management_group_scoped_error) => {
            match T::try_from_expanded_subscription_scoped(expanded) {
                Ok(x) => Ok(x),
                Err(subscription_scoped_error) => match T::try_from_expanded_unscoped(expanded) {
                    Ok(x) => Ok(x),
                    Err(unscoped_error) => {
                        bail!("Policy definition id parse failed.\n\nmanagement group scoped attempt: {management_group_scoped_error:?}\n\nsubscription scoped attempt: {subscription_scoped_error:?}\n\nunscoped attempt: {unscoped_error:?}")
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub enum ScopeError {
    Malformed,
    InvalidName,
    Unrecognized,
}
impl std::error::Error for ScopeError {}
impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ScopeError::Malformed => "malformed scope",
            ScopeError::InvalidName => "invalid name in scope",
            ScopeError::Unrecognized => "unrecognized scope kind",
        })
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ScopeImpl {
    ManagementGroup(ManagementGroupId),
    PolicyDefinition(PolicyDefinitionId),
    PolicySetDefinition(PolicySetDefinitionId),
    PolicyAssignment(PolicyAssignmentId),
    ResourceGroup(ResourceGroupId),
    RoleAssignment(RoleAssignmentId),
}
impl Scope for ScopeImpl {
    fn expanded_form(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(m) => m.expanded_form(),
            ScopeImpl::PolicyDefinition(p) => p.expanded_form(),
            ScopeImpl::PolicySetDefinition(p) => p.expanded_form(),
            ScopeImpl::PolicyAssignment(p) => p.expanded_form(),
            ScopeImpl::ResourceGroup(r) => r.expanded_form(),
            ScopeImpl::RoleAssignment(r) => r.expanded_form(),
        }
    }

    fn short_form(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(m) => m.short_form(),
            ScopeImpl::PolicyDefinition(p) => p.short_form(),
            ScopeImpl::PolicySetDefinition(p) => p.short_form(),
            ScopeImpl::PolicyAssignment(p) => p.short_form(),
            ScopeImpl::ResourceGroup(r) => r.short_name(),
            ScopeImpl::RoleAssignment(r) => r.short_name(),
        }
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        if let Ok(scope) = ManagementGroupId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ManagementGroup(scope));
        }
        if let Ok(scope) = PolicyDefinitionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyDefinition(scope));
        }
        if let Ok(scope) = PolicySetDefinitionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicySetDefinition(scope));
        }
        if let Ok(scope) = PolicyAssignmentId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyAssignment(scope));
        }
        Err(ScopeError::Unrecognized.into())
    }
}

impl FromStr for ScopeImpl {
    type Err = Error;

    fn from_str(s: &str) -> Result<ScopeImpl> {
        Self::try_from_expanded(s)
    }
}

impl std::fmt::Display for ScopeImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeImpl::ManagementGroup(x) => {
                f.write_fmt(format_args!("ManagementGroup({})", x.expanded_form()))
            }
            ScopeImpl::PolicyDefinition(x) => {
                f.write_fmt(format_args!("PolicyDefinition({})", x.expanded_form()))
            }
            ScopeImpl::PolicySetDefinition(x) => {
                f.write_fmt(format_args!("PolicySetDefinition({})", x.expanded_form()))
            }
            ScopeImpl::PolicyAssignment(x) => {
                f.write_fmt(format_args!("PolicyAssignment({})", x.expanded_form()))
            }
            ScopeImpl::ResourceGroup(x) => {
                f.write_fmt(format_args!("ResourceGroup({})", x.expanded_form()))
            }
            ScopeImpl::RoleAssignment(x) => {
                f.write_fmt(format_args!("RoleAssignment({})", x.expanded_form()))
            }
        }
    }
}
