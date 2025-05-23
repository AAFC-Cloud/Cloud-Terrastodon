use crate::prelude::ResourceGroupName;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::prelude::strip_prefix_get_slug_and_leading_slashed_remains;
use crate::scopes::HasPrefix;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::strip_prefix_case_insensitive;
use crate::slug::HasSlug;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::hash::Hash;
use std::str::FromStr;
use uuid::Uuid;

pub const RESOURCE_GROUP_ID_PREFIX: &str = "/resourceGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceGroupId {
    pub subscription_id: SubscriptionId,
    pub resource_group_name: ResourceGroupName,
}
impl ResourceGroupId {
    pub fn new(
        subscription_id: impl Into<SubscriptionId>,
        resource_group_name: impl Into<ResourceGroupName>,
    ) -> ResourceGroupId {
        ResourceGroupId {
            subscription_id: subscription_id.into(),
            resource_group_name: resource_group_name.into(),
        }
    }
}
impl HasSlug for ResourceGroupId {
    type Name = ResourceGroupName;

    fn name(&self) -> &Self::Name {
        &self.resource_group_name
    }
}
impl SubscriptionScoped for ResourceGroupId {
    fn subscription_id(&self) -> &SubscriptionId {
        &self.subscription_id
    }
}

impl std::fmt::Display for ResourceGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded_form().as_str())
    }
}

impl HasPrefix for ResourceGroupId {
    fn get_prefix() -> &'static str {
        RESOURCE_GROUP_ID_PREFIX
    }
}
impl TryFromSubscriptionScoped for ResourceGroupId {
    unsafe fn new_subscription_scoped_unchecked(
        _expanded: &str,
        subscription_id: SubscriptionId,
        name: Self::Name,
    ) -> Self {
        Self {
            subscription_id,
            resource_group_name: name,
        }
    }
}

impl FromStr for ResourceGroupId {
    type Err = eyre::Error;

    fn from_str(expanded_form: &str) -> Result<Self, Self::Err> {
        // /subscriptions/0000-000-00000/resourceGroups/My-Resource-Group/optional/other/stuff

        // "0000-000-0000", maybe "/resourceGroups/My-Resource-Group/optional/other/stuff"
        let (sub_id, remaining) = strip_prefix_get_slug_and_leading_slashed_remains(
            expanded_form,
            SUBSCRIPTION_ID_PREFIX,
        )
        .context(format!(
            "Tried to parse {:?} as resource group id, but prefix {} was missing",
            expanded_form, SUBSCRIPTION_ID_PREFIX
        ))?;

        // "0000-000-0000" => Uuid{0000-000-0000}
        let sub_id = sub_id.parse::<Uuid>().context(format!("Tried to parse {:?} as a resource group id, but the subscription id {:?} isn't a valid guid", expanded_form, sub_id))?;
        let sub_id = SubscriptionId::new(sub_id);

        // maybe "/resourceGroups/..." => Some("/resourceGroups/...")
        let Some(remaining) = remaining else {
            bail!(
                "Tried to parse {:?} as resource group id, but the stuff after the subscription id was missing",
                expanded_form
            )
        };

        // "My-Resource-Group", "optional/other/stuff"
        let (rg_name, remaining) = strip_prefix_get_slug_and_leading_slashed_remains(
            remaining,
            RESOURCE_GROUP_ID_PREFIX,
        )
        .context(format!(
            "Tried to parse {:?} as a resource group id, but chunk {:?} was missing prefix {:?}",
            expanded_form, remaining, RESOURCE_GROUP_ID_PREFIX
        ))?;
        let rg_name: ResourceGroupName = rg_name.parse()?;

        if let Some(remaining) = remaining {
            bail!(
                "Tried to parse {:?} as a resource group id, but encountered unexpected trailing remains {:?}",
                expanded_form,
                remaining
            );
        }

        Ok(ResourceGroupId::new(sub_id, rg_name.to_owned()))
    }
}

impl Scope for ResourceGroupId {
    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.subscription_id.expanded_form(),
            RESOURCE_GROUP_ID_PREFIX,
            self.resource_group_name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        let (subscription, resource_group) =
            strip_prefix_get_slug_and_leading_slashed_remains(expanded, SUBSCRIPTION_ID_PREFIX)?;
        let Some(resource_group) = resource_group else {
            bail!(
                "Could not create resource group id from {expanded:?}, extracted subscription {subscription} but found no resource group afterwards"
            );
        };
        let subscription_id = subscription.parse()?;
        let resource_group_name =
            strip_prefix_case_insensitive(resource_group, ResourceGroupId::get_prefix())?;
        let resource_group_name = resource_group_name.parse()?;
        Ok(ResourceGroupId {
            subscription_id,
            resource_group_name,
        })
    }

    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ResourceGroup(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ResourceGroup
    }
}

impl Serialize for ResourceGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}
