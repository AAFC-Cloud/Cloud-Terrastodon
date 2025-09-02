use crate::management_groups::MANAGEMENT_GROUP_ID_PREFIX;
use crate::management_groups::ManagementGroupId;
use crate::prelude::ContainerRegistryId;
use crate::prelude::KeyVaultId;
use crate::prelude::PolicyAssignmentId;
use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicySetDefinitionId;
use crate::prelude::RESOURCE_GROUP_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceId;
use crate::prelude::ResourceTagsId;
use crate::prelude::RoleAssignmentId;
use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleEligibilityScheduleId;
use crate::prelude::RoleManagementPolicyAssignmentId;
use crate::prelude::RoleManagementPolicyId;
use crate::prelude::RouteTableId;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::prelude::StorageAccountId;
use crate::prelude::SubnetId;
use crate::prelude::SubscriptionId;
use crate::prelude::TestResourceId;
use crate::prelude::VirtualNetworkId;
use crate::prelude::VirtualNetworkPeeringId;
use crate::slug::HasSlug;
use crate::slug::Slug;
use clap::ValueEnum;
use cloud_terrastodon_azure_resource_types::ResourceType;
use compact_str::CompactString;
use compact_str::ToCompactString;
use eyre::Context;
use eyre::bail;
use eyre::eyre;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Visitor;
use serde::de::{self};
use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;
use std::str::pattern::Pattern;

pub trait Scope: Sized + FromStr {
    type Err = <Self as FromStr>::Err;
    fn expanded_form(&self) -> String;
    fn short_form(&self) -> String {
        self.expanded_form()
            .rsplit_once('/')
            .map(|x| x.1.to_owned())
            .unwrap_or_else(|| self.expanded_form())
    }

    // TODO: replace String and CompactString usage with something like this
    // pub struct ExpandedFormScope(CompactString);
    // replace this fn with inheriting from TryFrom<ExpandedFormScope>
    fn try_from_expanded(expanded: &str) -> Result<Self, <Self as Scope>::Err>;

    // TODO: maybe replace with Into impl to avoid always cloning
    fn as_scope_impl(&self) -> ScopeImpl;

    fn kind(&self) -> ScopeImplKind;
}
impl Scope for CompactString {
    fn expanded_form(&self) -> String {
        self.to_string()
    }

    fn try_from_expanded(expanded: &str) -> Result<Self, Infallible> {
        Ok(expanded.to_compact_string())
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::Unknown(self.to_owned())
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Unknown
    }
}

pub trait AsScope {
    fn as_scope(&self) -> &impl Scope;
}
impl<T> AsScope for T
where
    T: Scope,
{
    fn as_scope(&self) -> &impl Scope {
        self
    }
}
pub trait NameValidatable {
    fn validate_name(name: &str) -> eyre::Result<()>;
}
pub trait HasPrefix {
    fn get_prefix() -> &'static str;
}

pub fn strip_prefix_case_insensitive<'a>(expanded: &'a str, prefix: &str) -> eyre::Result<&'a str> {
    if !prefix.to_lowercase().is_prefix_of(&expanded.to_lowercase()) {
        return Err(ScopeError::Malformed).context(format!(
            "String {expanded:?} must begin with {prefix:?} (case insensitive)"
        ));
    }
    let remaining = &expanded[prefix.len()..];
    Ok(remaining)
}

pub fn strip_suffix_case_insensitive<'a>(expanded: &'a str, suffix: &str) -> eyre::Result<&'a str> {
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
) -> eyre::Result<(&'a str, Option<&'a str>)> {
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

pub fn get_provider_and_resource_type_and_resource_and_remaining(
    expanded: &str,
) -> eyre::Result<(ResourceType, &str, &str)> {
    // /providers/Microsoft.KeyVault/vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Network/bastionHosts/my-bst/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Storage/storageAccounts/mystorage/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Network/virtualNetworks/my-vnet/providers/Microsoft.Authorization/roleAssignments/0000
    // /providers/Microsoft.Compute/virtualMachines/my-vm/providers/Microsoft.Authorization/roleAssignments/0000
    let remaining = expanded;
    let remaining = strip_prefix_case_insensitive(remaining, "/providers/")?;
    // Microsoft.KeyVault/vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (provider, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing provider"))?;
    // vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (resource_type, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing resource type"))?;
    let provider_and_resource_type = &expanded
        ["/providers/".len()..provider.len() + resource_type.len() + "/providers/".len() + 1];
    let resource_type = ResourceType::from_str(provider_and_resource_type)?;
    // my-vault/providers/Microsoft.Authorization/roleAssignments/0000
    let (resource, remaining) = remaining
        .split_once('/')
        .ok_or_else(|| eyre!("Missing resource name"))?;
    // providers/Microsoft.Authorization/roleAssignments/0000
    let remaining = &expanded[expanded.len() - remaining.len() - 1..];
    // /providers/Microsoft.Authorization/roleAssignments/0000

    Ok((resource_type, resource, remaining))
}

#[cfg(test)]
mod test {
    use super::get_provider_and_resource_type_and_resource_and_remaining;
    use cloud_terrastodon_azure_resource_types::ResourceType;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let x = "/providers/Microsoft.KeyVault/vaults/my-vault/providers/Microsoft.Authorization/roleAssignments/0000";
        let (resource_type, name, remaining) =
            get_provider_and_resource_type_and_resource_and_remaining(x)?;
        assert_eq!(
            resource_type,
            ResourceType::MICROSOFT_DOT_KEYVAULT_SLASH_VAULTS
        );
        assert_eq!(name, "my-vault");
        assert_eq!(
            remaining,
            "/providers/Microsoft.Authorization/roleAssignments/0000"
        );
        Ok(())
    }
}

pub trait TryFromUnscoped
where
    Self: Sized + HasPrefix + HasSlug,
{
    fn try_from_expanded_unscoped(expanded_unscoped: &str) -> eyre::Result<Self> {
        // Get name without prefix
        let prefix = Self::get_prefix();
        let name = strip_prefix_case_insensitive(expanded_unscoped, prefix)?;
        let name = <<Self as HasSlug>::Name>::try_new(name)?;
        unsafe { Ok(Self::new_unscoped_unchecked(expanded_unscoped, name)) }
    }
    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_unscoped_unchecked(_expanded: &str, name: Self::Name) -> Self;
}

pub trait TryFromResourceGroupScoped
where
    Self: Sized + HasPrefix + HasSlug,
{
    fn try_from_expanded_resource_group_scoped(expanded: &str) -> eyre::Result<Self> {
        let (subscription, remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let subscription_id = subscription.parse()?;
        let Some(remaining) = remaining else {
            bail!(
                "Could not create resource-group-scoped id from {expanded:?}, extracted subscription {subscription} but found no content afterwards"
            );
        };

        let (resource_group, remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(remaining, RESOURCE_GROUP_ID_PREFIX)?;
        let resource_group_name = resource_group.parse()?;
        let resource_group_id = ResourceGroupId {
            subscription_id,
            resource_group_name,
        };
        let Some(remaining) = remaining else {
            bail!(
                "Could not create resource-group-scoped id from {expanded:?}, extracted resource group {resource_group_id} but found no content afterwards"
            );
        };
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        let name = <<Self as HasSlug>::Name>::try_new(name)?;
        unsafe {
            Ok(Self::new_resource_group_scoped_unchecked(
                expanded,
                resource_group_id,
                name,
            ))
        }
    }

    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self;
}

pub trait TryFromResourceScoped
where
    Self: Sized + HasPrefix + HasSlug,
{
    fn try_from_expanded_resource_scoped(expanded: &str) -> eyre::Result<Self> {
        let (subscription_id, remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let subscription_id = subscription_id.parse()?;
        let Some(remaining) = remaining else {
            bail!(
                "Could not create resource-scoped id from {expanded:?}, extracted subscription {subscription_id} but found no content afterwards"
            );
        };

        let (resource_group_name, remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(remaining, RESOURCE_GROUP_ID_PREFIX)?;
        let resource_group_name = resource_group_name.parse()?;
        let resource_group_id = ResourceGroupId {
            subscription_id,
            resource_group_name,
        };
        let Some(remaining) = remaining else {
            bail!(
                "Could not create resource-scoped id from {expanded:?}, extracted resource group {resource_group_id} but found no content afterwards"
            );
        };
        let (resource_type, resource_name, remaining) =
            get_provider_and_resource_type_and_resource_and_remaining(remaining)?;
        let resource_id = ResourceId {
            resource_group_id,
            resource_type,
            resource_name: resource_name.to_compact_string(),
        };
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
        let name = <<Self as HasSlug>::Name>::try_new(name)?;
        unsafe {
            Ok(Self::new_resource_scoped_unchecked(
                expanded,
                resource_id,
                name,
            ))
        }
    }

    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        name: Self::Name,
    ) -> Self;
}

pub trait TryFromSubscriptionScoped
where
    Self: Sized + HasPrefix + HasSlug,
{
    fn try_from_expanded_subscription_scoped(expanded: &str) -> eyre::Result<Self> {
        let (subscription, remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let Some(remaining) = remaining else {
            bail!(
                "Could not create subscription-scoped id from {expanded:?}, extracted subscription {subscription} but found no content afterwards"
            );
        };
        let subscription_id = subscription.parse()?;
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        let name = <<Self as HasSlug>::Name>::try_new(name)?;
        unsafe {
            Ok(Self::new_subscription_scoped_unchecked(
                expanded,
                subscription_id,
                name,
            ))
        }
    }

    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self;
}
pub trait TryFromManagementGroupScoped
where
    Self: Sized + HasPrefix + HasSlug,
{
    fn try_from_expanded_management_group_scoped(expanded: &str) -> eyre::Result<Self> {
        let (management_group, remaining) = strip_prefix_get_slug_and_leading_slashed_remains(
            expanded,
            MANAGEMENT_GROUP_ID_PREFIX,
        )?;
        let management_group_id = management_group.parse()?;
        let Some(remaining) = remaining else {
            bail!(
                "Could not create management-group-scoped id from {expanded:?}, extracted management group {management_group} but found no content afterwards"
            );
        };
        let name = strip_prefix_case_insensitive(remaining, Self::get_prefix())?;
        let name = <<Self as HasSlug>::Name>::try_new(name)?;
        unsafe {
            Ok(Self::new_management_group_scoped_unchecked(
                expanded,
                management_group_id,
                name,
            ))
        }
    }
    /// # Safety
    ///
    /// The try_from methods should be used instead
    unsafe fn new_management_group_scoped_unchecked(
        _expanded: &str,
        management_group_id: ManagementGroupId,
        name: Self::Name,
    ) -> Self;
}

pub trait TryFromVirtualNetworkScoped
where
    Self: Sized,
{
    fn try_from_virtual_network_scoped(
        virtual_network_id: &VirtualNetworkId,
        name: &str,
    ) -> eyre::Result<Self>;
}

pub fn try_from_expanded_resource_container_scoped<T>(expanded: &str) -> eyre::Result<T>
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
                                        "Expanded resource container scoped parse failed for ",
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
pub fn try_from_expanded_hierarchy_scoped<T>(expanded: &str) -> eyre::Result<T>
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
pub trait ManagementGroupScoped {
    fn management_group_id(&self) -> &ManagementGroupId;
}
pub trait SubscriptionScoped {
    fn subscription_id(&self) -> &SubscriptionId;
}
pub trait ResourceGroupScoped {
    fn resource_group_id(&self) -> &ResourceGroupId;
}
pub trait ResourceScoped {
    fn resource_id(&self) -> &ResourceId;
}
impl<T: ResourceScoped> ResourceGroupScoped for T {
    fn resource_group_id(&self) -> &ResourceGroupId {
        &self.resource_id().resource_group_id
    }
}
impl<T: ResourceGroupScoped> SubscriptionScoped for T {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.resource_group_id().subscription_id
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
    VirtualNetwork,
    Subnet,
    ContainerRegistry,
    Subscription,
    Test,
    ResourceTags,
    Resource,
    RouteTable,
    VirtualNetworkPeering,
    KeyVault,
    Unknown,
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
    VirtualNetwork(VirtualNetworkId),
    Subnet(SubnetId),
    ResourceTags(ResourceTagsId),
    ContainerRegistry(ContainerRegistryId),
    Resource(ResourceId),
    RouteTable(RouteTableId),
    VirtualNetworkPeering(VirtualNetworkPeeringId),
    KeyVault(KeyVaultId),
    Unknown(CompactString),
}
impl Scope for ScopeImpl {
    type Err = Infallible;
    fn expanded_form(&self) -> String {
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
            ScopeImpl::VirtualNetwork(id) => id.expanded_form(),
            ScopeImpl::Subnet(id) => id.expanded_form(),
            ScopeImpl::ResourceTags(id) => id.expanded_form(),
            ScopeImpl::Resource(id) => id.expanded_form(),
            ScopeImpl::Unknown(id) => id.to_string(),
            ScopeImpl::RouteTable(id) => id.expanded_form(),
            ScopeImpl::VirtualNetworkPeering(id) => id.expanded_form(),
            ScopeImpl::ContainerRegistry(id) => id.expanded_form(),
            ScopeImpl::KeyVault(id) => id.expanded_form(),
        }
    }

    fn short_form(&self) -> String {
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
            ScopeImpl::VirtualNetwork(id) => id.short_form(),
            ScopeImpl::Subnet(id) => id.short_form(),
            ScopeImpl::ResourceTags(id) => id.short_form(),
            ScopeImpl::Resource(id) => id.short_form(),
            ScopeImpl::ContainerRegistry(id) => id.short_form(),
            ScopeImpl::RouteTable(id) => id.short_form(),
            ScopeImpl::VirtualNetworkPeering(id) => id.short_form(),
            ScopeImpl::Unknown(id) => id.to_string(),
            ScopeImpl::KeyVault(id) => id.short_form(),
        }
    }

    fn try_from_expanded(expanded: &str) -> Result<Self, <Self as Scope>::Err> {
        if let Ok(id) = ResourceGroupId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ResourceGroup(id));
        }
        if let Ok(id) = SubscriptionId::from_str(expanded) {
            return Ok(ScopeImpl::Subscription(id));
        }
        if let Ok(id) = ManagementGroupId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ManagementGroup(id));
        }
        if let Ok(id) = StorageAccountId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::StorageAccount(id));
        }
        if let Ok(id) = VirtualNetworkId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::VirtualNetwork(id));
        }
        if let Ok(id) = SubnetId::try_from(expanded) {
            return Ok(ScopeImpl::Subnet(id));
        }
        if let Ok(id) = ContainerRegistryId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::ContainerRegistry(id));
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
        if let Ok(id) = RouteTableId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::RouteTable(id));
        }
        if let Ok(id) = VirtualNetworkPeeringId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::VirtualNetworkPeering(id));
        }
        if let Ok(id) = ResourceId::try_from_expanded(expanded) {
            return Ok(ScopeImpl::Resource(id));
        }
        Ok(ScopeImpl::Unknown(expanded.to_compact_string()))
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
            ScopeImpl::VirtualNetwork(_) => ScopeImplKind::VirtualNetwork,
            ScopeImpl::Subnet(_) => ScopeImplKind::Subnet,
            ScopeImpl::RoleEligibilitySchedule(_) => ScopeImplKind::RoleEligibilitySchedule,
            ScopeImpl::RoleManagementPolicyAssignment(_) => {
                ScopeImplKind::RoleManagementPolicyAssignment
            }
            ScopeImpl::RoleManagementPolicy(_) => ScopeImplKind::RoleManagementPolicyAssignment,
            ScopeImpl::Unknown(_) => ScopeImplKind::Unknown,
            ScopeImpl::ResourceTags(_) => ScopeImplKind::ResourceTags,
            ScopeImpl::ContainerRegistry(_) => ScopeImplKind::ContainerRegistry,
            ScopeImpl::RouteTable(_) => ScopeImplKind::RouteTable,
            ScopeImpl::VirtualNetworkPeering(_) => ScopeImplKind::VirtualNetworkPeering,
            ScopeImpl::Resource(_) => ScopeImplKind::Resource,
            ScopeImpl::KeyVault(_) => ScopeImplKind::KeyVault,
        }
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        self.clone()
    }
}

impl FromStr for ScopeImpl {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<ScopeImpl, Infallible> {
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
            ScopeImpl::VirtualNetworkPeering(x) => {
                f.write_fmt(format_args!("VirtualNetworkPeering({})", x.short_form()))
            }
            ScopeImpl::VirtualNetwork(x) => {
                f.write_fmt(format_args!("VirtualNetwork({})", x.short_form()))
            }
            ScopeImpl::Subnet(x) => f.write_fmt(format_args!("Subnet({})", x.short_form())),
            ScopeImpl::ContainerRegistry(x) => {
                f.write_fmt(format_args!("ContainerRegistry({})", x.short_form()))
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
            ScopeImpl::RouteTable(x) => f.write_fmt(format_args!("RouteTable({})", x.short_form())),
            ScopeImpl::Resource(x) => f.write_fmt(format_args!(
                "Resource({}/{})",
                x.resource_type,
                x.short_form()
            )),
            ScopeImpl::Unknown(x) => f.write_fmt(format_args!("Raw({x})")),
            ScopeImpl::KeyVault(x) => f.write_fmt(format_args!("KeyVault({})", x.short_form())),
        }
    }
}
impl<T> From<T> for ScopeImpl
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let Ok(scope) = ScopeImpl::try_from_expanded(value.as_ref());
        scope
    }
}

impl Serialize for ScopeImpl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
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
        ScopeImpl::try_from_expanded(value).map_err(|e| E::custom(format!("{e:#?}")))
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
    use super::*;
    use itertools::Itertools;
    use uuid::Uuid;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let scope = ScopeImpl::TestResource(TestResourceId::new("bruh"));
        let expected = format!("{:?}", scope.expanded_form());
        assert_eq!(serde_json::to_string(&scope)?, expected);
        Ok(())
    }
    #[test]
    fn it_works2() -> eyre::Result<()> {
        let scope = ScopeImpl::Subscription(SubscriptionId::new(Uuid::nil()));
        let expected = format!("{:?}", scope.expanded_form());
        assert_eq!(serde_json::to_string(&scope)?, expected);
        Ok(())
    }
    #[test]
    fn rg_id_equality_case_silly() -> eyre::Result<()> {
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
    fn rg_id_equality_negative() -> eyre::Result<()> {
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
    fn test_strip_suffix() -> eyre::Result<()> {
        let x = "abcde";
        let suffix = "CDE";
        let stripped = strip_suffix_case_insensitive(&x, suffix)?;
        assert_eq!("ab", stripped);
        Ok(())
    }
}
