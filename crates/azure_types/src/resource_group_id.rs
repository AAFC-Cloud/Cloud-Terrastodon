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
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::hash::Hash;
use std::str::FromStr;
use uuid::Uuid;

pub const RESOURCE_GROUP_ID_PREFIX: &str = "/resourceGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Arbitrary)]
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
    pub fn try_new<S, N>(subscription_id: S, resource_group_name: N) -> Result<Self>
    where
        S: TryInto<SubscriptionId>,
        S::Error: Into<eyre::Error>,
        N: TryInto<ResourceGroupName>,
        N::Error: Into<eyre::Error>,
    {
        let subscription_id = subscription_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert subscription_id")?;
        let resource_group_name = resource_group_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_name")?;
        Ok(ResourceGroupId {
            subscription_id,
            resource_group_name,
        })
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
            "Tried to parse {expanded_form:?} as resource group id, but prefix {SUBSCRIPTION_ID_PREFIX} was missing"
        ))?;

        // "0000-000-0000" => Uuid{0000-000-0000}
        let sub_id = sub_id.parse::<Uuid>().context(format!("Tried to parse {expanded_form:?} as a resource group id, but the subscription id {sub_id:?} isn't a valid guid"))?;
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
            "Tried to parse {expanded_form:?} as a resource group id, but chunk {remaining:?} was missing prefix {RESOURCE_GROUP_ID_PREFIX:?}"
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
impl TryFrom<&str> for ResourceGroupId {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ResourceGroupId::from_str(value)
    }
}
impl TryFrom<String> for ResourceGroupId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
impl TryFrom<&String> for ResourceGroupId {
    type Error = eyre::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
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

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
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
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::ResourceGroupId;
    use uuid::Uuid;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        // a random guid
        let _id =
            ResourceGroupId::try_new("d8e27b8a-2513-45f5-9ae7-10fcb50e645e", "My-Resource-Group")?;
        Ok(())
    }

    #[test]
    pub fn easy_construct() -> eyre::Result<()> {
        let subscription_id = Uuid::nil().to_string();
        let resource_group_name = "My-RG".to_string();
        let id = ResourceGroupId::try_new(subscription_id, resource_group_name)?;
        println!("{id}");
        Ok(())
    }
}
