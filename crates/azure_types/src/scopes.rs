use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;
use crate::management_groups::ManagementGroupId;
use crate::policy_assignments::PolicyAssignmentId;
use crate::policy_definitions::PolicyDefinitionId;
use crate::policy_set_definitions::PolicySetDefinitionId;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceTagsId;
use crate::prelude::RoleAssignmentId;
use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleManagementPolicyAssignmentId;
use crate::prelude::RoleManagementPolicyId;
use crate::prelude::StorageAccountId;
use crate::prelude::SubscriptionId;
use crate::prelude::TestResourceId;
use crate::prelude::RESOURCE_GROUP_ID_PREFIX;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::role_eligibility_schedules::RoleEligibilityScheduleId;
use clap::ValueEnum;
use core::panic;
use eyre::Context;
use eyre::Error;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Visitor;
use serde::de::{self};
use std::fmt;
use std::str::FromStr;
use std::str::pattern::Pattern;

pub trait HasName {
    fn name(&self) -> &str;
}

pub trait Scope: Sized {
    fn expanded_form(&self) -> &str;
    fn short_form(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .map(|x| x.1)
            .unwrap_or_else(|| self.expanded_form())
    }
    fn try_from_expanded(expanded: &str) -> Result<Self>;
    fn as_scope(&self) -> ScopeImpl;
    fn kind(&self) -> ScopeImplKind;
}
pub trait HasScope {
    fn scope(&self) -> &impl Scope;
}
impl<T> HasScope for T
where
    T: Scope,
{
    fn scope(&self) -> &impl Scope {
        self
    }
}

pub trait NameValidatable {
    fn validate_name(name: &str) -> Result<()>;
}
pub trait HasPrefix {
    fn get_prefix() -> &'static str;
}

pub fn strip_prefix_case_insensitive<'a>(expanded: &'a str, prefix: &str) -> Result<&'a str> {
    if !prefix.to_lowercase().is_prefix_of(&expanded.to_lowercase()) {
        return Err(ScopeError::Malformed).context(format!(
            "String {expanded:?} must begin with {prefix:?} (case insensitive)"
        ));
    }
    let remaining = &expanded[prefix.len()..];
    Ok(remaining)
}

pub fn strip_suffix_case_insensitive<'a>(expanded: &'a str, suffix: &str) -> Result<&'a str> {
    if !suffix.to_lowercase().is_suffix_of(&expanded.to_lowercase()) {
        return Err(ScopeError::Malformed).context(format!(
            "String {expanded:?} must end with {suffix:?} (case insensitive)"
        ));
    }
    let remaining = &expanded[..suffix.len() - 1];
    Ok(remaining)
}

pub fn strip_prefix_get_slug_and_leading_slashed_remains<'a>(
    expanded: &'a str,
    prefix: &str,
) -> Result<(&'a str, Option<&'a str>)> {
    // /subscription/abc/resourceGroups/def
    // /subscription/abc
    // Remove prefix
    let remaining = strip_prefix_case_insensitive(expanded, prefix)?;

    // abc/resourceGroups/def
    // abc
    // Capture slug and remains
    if !remaining.contains('/') {
        // abc, None
        return Ok((remaining, None));
    }

    // abc, resourceGroups/def
    let (slug, remaining) = remaining.split_once('/').unwrap();
    // Unstrip leading slash
    let remaining = &expanded[expanded.len() - remaining.len() - 1..];

    // abc, /resourceGroups/def
    Ok((slug, Some(remaining)))
}

fn strip_prefix_and_slug_leaving_slash<'a>(expanded: &'a str, prefix: &str) -> Result<&'a str> {
    // /subscription/abc/resourceGroups/def
    // Remove prefix
    let remaining = strip_prefix_case_insensitive(expanded, prefix)?;

    // abc/resourceGroups/def
    // Remove slug
    let Some((_slug, remaining)) = remaining.split_once('/') else {
        return Err(ScopeError::Malformed).context(format!(
            "String {expanded:?} must contain a slash after the prefix {prefix:?}"
        ));
    };

    // resourceGroups/def
    // Unstrip leading slash
    let remaining = &expanded[expanded.len() - remaining.len() - 1..];

    // /resourceGroups/def
    Ok(remaining)
}

fn strip_provider_and_resource(expanded: &str) -> Result<&str> {
    // /providers/Microsoft.KeyVault/vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Network/bastionHosts/my-bst/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Network/virtualNetworks/my-vnet/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Compute/virtualMachines/my-vm/providers/Microsoft.Authorization/roleAssignments/0000
    let remaining = expanded;
    let remaining = strip_prefix_case_insensitive(remaining, "/providers/")?;
    // Microsoft.KeyVault/vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (_, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing provider"))?;
    // vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (_, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing resource type"))?;
    // my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (_, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing resource name"))?;
    // providers/Microsoft.Authorization/roleAssignments/0000
    let remaining = &expanded[expanded.len() - remaining.len() - 1..];
    // /providers/Microsoft.Authorization/roleAssignments/0000

    Ok(remaining)
}

pub trait TryFromUnscoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_unscoped(expanded_unscoped: &str) -> Result<Self> {
        // Get name without prefix
        let prefix = Self::get_prefix();
        let name = strip_prefix_case_insensitive(expanded_unscoped, prefix)?;
        Self::validate_name(name).context("validating name")?;

        unsafe { Ok(Self::new_unscoped_unchecked(expanded_unscoped)) }
    }
    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self;
}

pub trait TryFromResourceGroupScoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_resource_group_scoped(expanded: &str) -> Result<Self> {
        let remaining = strip_prefix_and_slug_leaving_slash(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let remaining = strip_prefix_and_slug_leaving_slash(remaining, RESOURCE_GROUP_ID_PREFIX)?;
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        Self::validate_name(name).context("validating name")?;
        unsafe { Ok(Self::new_resource_group_scoped_unchecked(expanded)) }
    }

    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self;
}

pub trait TryFromResourceScoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_resource_scoped(expanded: &str) -> Result<Self> {
        let remaining = strip_prefix_and_slug_leaving_slash(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let remaining = strip_prefix_and_slug_leaving_slash(remaining, RESOURCE_GROUP_ID_PREFIX)?;
        let remaining = strip_provider_and_resource(remaining)?;
        // the resource could have a subresource, like a subnet on a vnet
        // now we search from the right
        let prefix = Self::get_prefix();
        let prefix_pos = remaining
            .to_lowercase()
            .rfind(&prefix.to_lowercase())
            .ok_or_else(|| {
                eyre!("String {remaining:?} must contain {prefix} (case insensitive)")
            })?;
        let name = &remaining[prefix_pos + prefix.len()..];
        Self::validate_name(name).context("validating name")?;
        unsafe { Ok(Self::new_resource_scoped_unchecked(expanded)) }
    }

    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_resource_scoped_unchecked(expanded: &str) -> Self;
}

pub trait TryFromSubscriptionScoped
where
    Self: Sized + NameValidatable + HasPrefix,
{
    fn try_from_expanded_subscription_scoped(expanded: &str) -> Result<Self> {
        let remaining = strip_prefix_and_slug_leaving_slash(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        Self::validate_name(name).context("validating name")?;
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
        let remaining = strip_prefix_and_slug_leaving_slash(expanded, MANAGEMENT_GROUP_ID_PREFIX)?;
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        Self::validate_name(name).context("validating name")?;
        unsafe { Ok(Self::new_management_group_scoped_unchecked(expanded)) }
    }
    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self;
}

pub fn try_from_expanded_resource_container_scoped<T>(expanded: &str) -> Result<T>
where
    T: TryFromUnscoped
        + TryFromManagementGroupScoped
        + TryFromSubscriptionScoped
        + TryFromResourceGroupScoped,
{
    match T::try_from_expanded_management_group_scoped(expanded) {
        Ok(x) => Ok(x),
        Err(management_group_scoped_error) => {
            match T::try_from_expanded_subscription_scoped(expanded) {
                Ok(x) => Ok(x),
                Err(subscription_scoped_error) => {
                    match T::try_from_expanded_resource_group_scoped(expanded) {
                        Ok(x) => Ok(x),
                        Err(resource_group_scoped_error) => {
                            match T::try_from_expanded_unscoped(expanded) {
                                Ok(x) => Ok(x),
                                Err(unscoped_error) => {
                                    bail!(
                                        "{}\n{:?}\n========\n{}\n{:?}\n\n{}\n{:?}\n\n{}\n{:?}\n\n{}\n{:?}",
                                        "Hierarchy scoped parse failed for ",
                                        expanded,
                                        "management group scoped attempt: ",
                                        management_group_scoped_error,
                                        "subscription scoped attempt: ",
                                        subscription_scoped_error,
                                        "resource group scoped attempt: ",
                                        resource_group_scoped_error,
                                        "unscoped attempt: ",
                                        unscoped_error
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
pub fn try_from_expanded_hierarchy_scoped<T>(expanded: &str) -> Result<T>
where
    T: TryFromUnscoped
        + TryFromManagementGroupScoped
        + TryFromSubscriptionScoped
        + TryFromResourceGroupScoped
        + TryFromResourceScoped,
{
    match T::try_from_expanded_management_group_scoped(expanded) {
        Ok(x) => Ok(x),
        Err(management_group_scoped_error) => {
            match T::try_from_expanded_subscription_scoped(expanded) {
                Ok(x) => Ok(x),
                Err(subscription_scoped_error) => {
                    match T::try_from_expanded_resource_group_scoped(expanded) {
                        Ok(x) => Ok(x),
                        Err(resource_group_scoped_error) => {
                            match T::try_from_expanded_resource_scoped(expanded) {
                                Ok(x) => Ok(x),
                                Err(resource_scoped_error) => {
                                    match T::try_from_expanded_unscoped(expanded) {
                                        Ok(x) => Ok(x),
                                        Err(unscoped_error) => {
                                            bail!(
                                                "{}\n{:?}\n========\n{}\n{:?}\n\n{}\n{:?}\n\n{}\n{:?}\n\n{}\n{:?}\n\n{}\n{:?}",
                                                "Hierarchy scoped parse failed for ",
                                                expanded,
                                                "management group scoped attempt: ",
                                                management_group_scoped_error,
                                                "subscription scoped attempt: ",
                                                subscription_scoped_error,
                                                "resource group scoped attempt: ",
                                                resource_group_scoped_error,
                                                "resource scoped attempt: ",
                                                resource_scoped_error,
                                                "unscoped attempt: ",
                                                unscoped_error
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub trait Unscoped {}
pub trait ManagementGroupScoped: Scope {
    fn management_group_id(&self) -> ManagementGroupId {
        ManagementGroupId::from_name(
            strip_prefix_get_slug_and_leading_slashed_remains(
                self.expanded_form(),
                MANAGEMENT_GROUP_ID_PREFIX,
            )
            .expect("management group prefix should have been validated before construction")
            .0,
        )
    }
}
pub trait SubscriptionScoped: Scope {
    fn subscription_id(&self) -> SubscriptionId {
        SubscriptionId::new(
            strip_prefix_get_slug_and_leading_slashed_remains(
                self.expanded_form(),
                SUBSCRIPTION_ID_PREFIX,
            )
            .expect("subscription prefix should have been validated before construction")
            .0
            .parse()
            .expect("subscription scoped should have valid uuid as subscription slug"),
        )
    }
}
pub trait ResourceGroupScoped: SubscriptionScoped {
    fn resource_group_id(&self) -> ResourceGroupId {
        let expanded = self.expanded_form();
        let Ok((subscription_id, Some(remaining))) =
            strip_prefix_get_slug_and_leading_slashed_remains(expanded, SUBSCRIPTION_ID_PREFIX)
        else {
            panic!(
                "resource group id should have been validated before construction - expected subscription prefix with slug but got {expanded:?}"
            );
        };
        let Ok(subscription_id) = subscription_id.parse() else {
            panic!(
                "resource group id should have been validated before construction - subscription id malformed {subscription_id:?}"
            );
        };
        let Ok((resource_group_name, _)) =
            strip_prefix_get_slug_and_leading_slashed_remains(remaining, RESOURCE_GROUP_ID_PREFIX)
        else {
            panic!(
                "resource group id should have been validated before construction - expected resource group prefix with slug but got {expanded:?}"
            );
        };
        ResourceGroupId::new(&subscription_id, resource_group_name.to_string())
    }
}
pub trait ResourceScoped: ResourceGroupScoped {
    fn resource_id(&self) -> ResourceId {
        // /subscriptions/000/resourceGroups/abc/providers/Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let expanded = self.expanded_form();
        let remaining = strip_prefix_and_slug_leaving_slash(expanded, SUBSCRIPTION_ID_PREFIX)
            .expect("subscription id prefix should have been validated before construction");
        // /resourceGroups/abc/providers/Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let remaining = strip_prefix_and_slug_leaving_slash(remaining, RESOURCE_GROUP_ID_PREFIX)
            .expect("resource group id prefix should have been validated before construction");
        // /providers/Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let remaining = strip_prefix_and_slug_leaving_slash(remaining, "/providers/")
            .expect("resources should be prefixed with /providers/ after resource group stuff");
        // Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let remaining = remaining.split_once('/').expect("resources should have a subtype, /providers/Microsoft.Whatever/something_was_expected_here").1;
        // storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let remaining = remaining.split_once('/').expect("resources should have a subtype, /providers/Microsoft.Whatever/something_was_expected_here").1;
        // mystorage/providers/Microsoft.Authorization/roleAssignments/111
        let (_name, tail_len) = remaining
            .split_once('/')
            .map(|x| (x.0, x.1.len()))
            .unwrap_or((remaining, 0));
        // mystorage
        ResourceId::from_str(&expanded[0..expanded.len() - tail_len])
            .expect("should be able to construct expanded resource id")
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
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, ValueEnum)]
pub enum ScopeImplKind {
    ManagementGroup,
    RoleManagementPolicyAssignment,
    RoleManagementPolicy,
    PolicyDefinition,
    PolicySetDefinition,
    PolicyAssignment,
    ResourceGroup,
    RoleAssignment,
    RoleDefinition,
    RoleEligibilitySchedule,
    StorageAccount,
    Subscription,
    Test,
    ResourceTags,
    Raw,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ScopeImpl {
    ManagementGroup(ManagementGroupId),
    PolicyDefinition(PolicyDefinitionId),
    PolicySetDefinition(PolicySetDefinitionId),
    PolicyAssignment(PolicyAssignmentId),
    ResourceGroup(ResourceGroupId),
    RoleAssignment(RoleAssignmentId),
    RoleDefinition(RoleDefinitionId),
    RoleEligibilitySchedule(RoleEligibilityScheduleId),
    Subscription(SubscriptionId),
    TestResource(TestResourceId),
    RoleManagementPolicyAssignment(RoleManagementPolicyAssignmentId),
    RoleManagementPolicy(RoleManagementPolicyId),
    StorageAccount(StorageAccountId),
    ResourceTags(ResourceTagsId),
    Raw(ResourceId),
}
impl Scope for ScopeImpl {
    fn expanded_form(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(id) => id.expanded_form(),
            ScopeImpl::PolicyDefinition(id) => id.expanded_form(),
            ScopeImpl::PolicySetDefinition(id) => id.expanded_form(),
            ScopeImpl::PolicyAssignment(id) => id.expanded_form(),
            ScopeImpl::ResourceGroup(id) => id.expanded_form(),
            ScopeImpl::RoleAssignment(id) => id.expanded_form(),
            ScopeImpl::RoleDefinition(id) => id.expanded_form(),
            ScopeImpl::Subscription(id) => id.expanded_form(),
            ScopeImpl::TestResource(id) => id.expanded_form(),
            ScopeImpl::RoleEligibilitySchedule(id) => id.expanded_form(),
            ScopeImpl::RoleManagementPolicyAssignment(id) => id.expanded_form(),
            ScopeImpl::RoleManagementPolicy(id) => id.expanded_form(),
            ScopeImpl::StorageAccount(id) => id.expanded_form(),
            ScopeImpl::ResourceTags(id) => id.expanded_form(),
            ScopeImpl::Raw(id) => id.expanded_form(),
        }
    }

    fn short_form(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(id) => id.short_form(),
            ScopeImpl::PolicyDefinition(id) => id.short_form(),
            ScopeImpl::PolicySetDefinition(id) => id.short_form(),
            ScopeImpl::PolicyAssignment(id) => id.short_form(),
            ScopeImpl::ResourceGroup(id) => id.short_form(),
            ScopeImpl::RoleAssignment(id) => id.short_form(),
            ScopeImpl::RoleDefinition(id) => id.short_form(),
            ScopeImpl::Subscription(id) => id.short_form(),
            ScopeImpl::TestResource(id) => id.short_form(),
            ScopeImpl::RoleEligibilitySchedule(id) => id.short_form(),
            ScopeImpl::RoleManagementPolicyAssignment(id) => id.short_form(),
            ScopeImpl::RoleManagementPolicy(id) => id.short_form(),
            ScopeImpl::StorageAccount(id) => id.short_form(),
            ScopeImpl::ResourceTags(id) => id.short_form(),
            ScopeImpl::Raw(id) => id.short_form(),
        }
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        if let Ok(id) = ResourceGroupId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ResourceGroup(id));
        }
        if let Ok(id) = SubscriptionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::Subscription(id));
        }
        if let Ok(id) = ManagementGroupId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ManagementGroup(id));
        }
        if let Ok(id) = StorageAccountId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::StorageAccount(id));
        }
        if let Ok(id) = PolicyDefinitionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyDefinition(id));
        }
        if let Ok(id) = PolicySetDefinitionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicySetDefinition(id));
        }
        if let Ok(id) = PolicyAssignmentId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyAssignment(id));
        }
        if let Ok(id) = RoleAssignmentId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RoleAssignment(id));
        }
        if let Ok(id) = RoleDefinitionId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RoleDefinition(id));
        }
        if let Ok(id) = RoleEligibilityScheduleId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RoleEligibilitySchedule(id));
        }
        if let Ok(id) = RoleManagementPolicyAssignmentId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RoleManagementPolicyAssignment(id));
        }
        if let Ok(id) = RoleManagementPolicyId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RoleManagementPolicy(id));
        }
        if let Ok(id) = TestResourceId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::TestResource(id));
        }
        Err(ScopeError::Unrecognized.into())
    }

    fn kind(&self) -> ScopeImplKind {
        match self {
            ScopeImpl::ManagementGroup(_) => ScopeImplKind::ManagementGroup,
            ScopeImpl::PolicyDefinition(_) => ScopeImplKind::PolicyDefinition,
            ScopeImpl::PolicySetDefinition(_) => ScopeImplKind::PolicySetDefinition,
            ScopeImpl::PolicyAssignment(_) => ScopeImplKind::PolicyAssignment,
            ScopeImpl::ResourceGroup(_) => ScopeImplKind::ResourceGroup,
            ScopeImpl::RoleAssignment(_) => ScopeImplKind::RoleAssignment,
            ScopeImpl::RoleDefinition(_) => ScopeImplKind::RoleDefinition,
            ScopeImpl::Subscription(_) => ScopeImplKind::Subscription,
            ScopeImpl::TestResource(_) => ScopeImplKind::Test,
            ScopeImpl::StorageAccount(_) => ScopeImplKind::StorageAccount,
            ScopeImpl::RoleEligibilitySchedule(_) => ScopeImplKind::RoleEligibilitySchedule,
            ScopeImpl::RoleManagementPolicyAssignment(_) => {
                ScopeImplKind::RoleManagementPolicyAssignment
            }
            ScopeImpl::RoleManagementPolicy(_) => ScopeImplKind::RoleManagementPolicyAssignment,
            ScopeImpl::Raw(_) => ScopeImplKind::Raw,
            ScopeImpl::ResourceTags(_) => ScopeImplKind::ResourceTags,
        }
    }

    fn as_scope(&self) -> ScopeImpl {
        self.clone()
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
            ScopeImpl::RoleDefinition(x) => {
                f.write_fmt(format_args!("RoleDefinition({})", x.expanded_form()))
            }
            ScopeImpl::Subscription(x) => {
                f.write_fmt(format_args!("Subscription({})", x.short_form()))
            }
            ScopeImpl::TestResource(x) => {
                f.write_fmt(format_args!("TestResource({})", x.short_form()))
            }
            ScopeImpl::RoleEligibilitySchedule(x) => {
                f.write_fmt(format_args!("RoleEligibilitySchedule({})", x.short_form()))
            }
            ScopeImpl::StorageAccount(x) => {
                f.write_fmt(format_args!("StorageAccount({})", x.short_form()))
            }
            ScopeImpl::RoleManagementPolicyAssignment(x) => f.write_fmt(format_args!(
                "RoleManagementPolicyAssignment({})",
                x.short_form()
            )),
            ScopeImpl::RoleManagementPolicy(x) => {
                f.write_fmt(format_args!("RoleManagementPolicy({})", x.short_form()))
            }
            ScopeImpl::ResourceTags(x) => {
                f.write_fmt(format_args!("ResourceTags({})", x.short_form()))
            }
            ScopeImpl::Raw(x) => f.write_fmt(format_args!("Raw({})", x)),
        }
    }
}

impl Serialize for ScopeImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

struct ScopeImplVisitor;

impl<'de> Visitor<'de> for ScopeImplVisitor {
    type Value = ScopeImpl;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing an azure scope")
    }

    fn visit_str<E>(self, value: &str) -> Result<ScopeImpl, E>
    where
        E: de::Error,
    {
        ScopeImpl::try_from_expanded(value).map_err(|e| E::custom(format!("{e:#}")))
    }
}

impl<'de> Deserialize<'de> for ScopeImpl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ScopeImplVisitor)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let scope = ScopeImpl::TestResource(TestResourceId::new("bruh"));
        let expected = format!("{:?}", scope.expanded_form());
        assert_eq!(serde_json::to_string(&scope)?, expected);
        Ok(())
    }
    #[test]
    fn it_works2() -> Result<()> {
        let scope = ScopeImpl::Subscription(SubscriptionId::new(Uuid::nil()));
        let expected = format!("{:?}", scope.expanded_form());
        assert_eq!(serde_json::to_string(&scope)?, expected);
        Ok(())
    }
    #[test]
    fn rg_id_equality_case_silly() -> Result<()> {
        // Azure cannot be relied upon for consistency in resource group IDs
        let zero = Uuid::nil();
        let ids = [
            format!("/subscriptions/{zero}/resourceGroups/abc"),
            format!("/subscriptions/{zero}/ResourceGroups/abc"),
            format!("/subscriptions/{zero}/resourceGroups/Abc"),
            format!("/subscriptions/{zero}/ResourceGroups/Abc"),
            format!("/subscriptions/{zero}/resourceGroups/aBc"),
            format!("/subscriptions/{zero}/ResourceGroups/aBc"),
            format!("/subscriptions/{zero}/resourceGroups/abC"),
            format!("/subscriptions/{zero}/ResourceGroups/abC"),
        ];
        let mut parsed_ids = Vec::new();
        for id in ids {
            let x = id.parse::<ScopeImpl>()?;
            parsed_ids.push(x);
        }
        for x in parsed_ids.iter().combinations(2) {
            let left: &ScopeImpl = x[0];
            let right: &ScopeImpl = x[1];
            assert_eq!(left, right);
        }
        Ok(())
    }
    #[test]
    fn rg_id_equality_negative() -> Result<()> {
        // Azure cannot be relied upon for consistency in resource group IDs
        let zero = Uuid::nil();
        let ids = [
            format!("/subscriptions/{zero}/resourceGroups/abc"),
            format!("/subscriptions/{zero}/ResourceGroups/xyz"),
            format!("/subscriptions/{zero}/ResourceGroups/def"),
        ];
        let mut parsed_ids = Vec::new();
        for id in ids {
            let x = id.parse::<ScopeImpl>()?;
            parsed_ids.push(x);
        }
        for x in parsed_ids.iter().combinations(2) {
            let left: &ScopeImpl = x[0];
            let right: &ScopeImpl = x[1];
            assert_ne!(left, right);
        }
        Ok(())
    }
    #[test]
    fn test_strip_suffix() -> Result<()> {
        let x = "abcde";
        let suffix = "CDE";
        let stripped = strip_suffix_case_insensitive(&x, suffix)?;
        assert_eq!("ab", stripped);
        Ok(())
    }
}
