use crate::prelude::RESOURCE_GROUP_ID_PREFIX;
use crate::prelude::ResourceGroupId;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::prelude::VirtualNetworkId;
use crate::prelude::VirtualNetworkName;
use crate::prelude::VirtualNetworkPeeringName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::get_provider_and_resource_type_and_resource_and_remaining;
use crate::scopes::strip_prefix_case_insensitive;
use crate::scopes::strip_prefix_get_slug_and_leading_slashed_remains;
use crate::slug::HasSlug;
use crate::slug::Slug;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_resource_types::ResourceType;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct VirtualNetworkPeeringId {
    pub virtual_network_id: VirtualNetworkId,
    pub virtual_network_peering_name: VirtualNetworkPeeringName,
}
impl VirtualNetworkPeeringId {
    pub fn new(
        virtual_network_id: impl Into<VirtualNetworkId>,
        virtual_network_peering_name: impl Into<VirtualNetworkPeeringName>,
    ) -> VirtualNetworkPeeringId {
        VirtualNetworkPeeringId {
            virtual_network_id: virtual_network_id.into(),
            virtual_network_peering_name: virtual_network_peering_name.into(),
        }
    }

    pub fn try_new<R, N>(virtual_network_id: R, virtual_network_peering_name: N) -> Result<Self>
    where
        R: TryInto<VirtualNetworkId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<VirtualNetworkPeeringName>,
        N::Error: Into<eyre::Error>,
    {
        let virtual_network_id = virtual_network_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert virtual_network_id")?;
        let virtual_network_peering_name = virtual_network_peering_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert virtual_network_peering_name")?;
        Ok(VirtualNetworkPeeringId {
            virtual_network_id,
            virtual_network_peering_name,
        })
    }
}

impl HasSlug for VirtualNetworkPeeringId {
    type Name = VirtualNetworkPeeringName;

    fn name(&self) -> &Self::Name {
        &self.virtual_network_peering_name
    }
}
impl AsRef<VirtualNetworkPeeringName> for VirtualNetworkPeeringId {
    fn as_ref(&self) -> &VirtualNetworkPeeringName {
        &self.virtual_network_peering_name
    }
}

impl NameValidatable for VirtualNetworkPeeringId {
    fn validate_name(name: &str) -> Result<()> {
        VirtualNetworkPeeringName::try_new(name).map(|_| ())
    }
}
impl HasPrefix for VirtualNetworkPeeringId {
    fn get_prefix() -> &'static str {
        "/virtualNetworkPeerings/"
    }
}

impl Scope for VirtualNetworkPeeringId {
    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
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
        let (resource_type, vnet_name, remaining) =
            get_provider_and_resource_type_and_resource_and_remaining(remaining)?;
        if resource_type != ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS {
            bail!(
                "Expected resource type to be Microsoft.Network/virtualNetworks, but found {resource_type:?} in {expanded:?}"
            );
        }

        let virtual_network_name = VirtualNetworkName::try_from(vnet_name)?;
        let remaining = strip_prefix_case_insensitive(remaining, "/virtualnetworkpeerings/")?;
        let virtual_network_peering_name = VirtualNetworkPeeringName::try_from(remaining)?;
        let virtual_network_id = VirtualNetworkId::new(resource_group_id, virtual_network_name);
        Ok(VirtualNetworkPeeringId {
            virtual_network_id,
            virtual_network_peering_name,
        })
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}/virtualNetworkPeerings/{}",
            self.virtual_network_id.expanded_form(),
            self.virtual_network_peering_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::VirtualNetworkPeering
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::VirtualNetworkPeering(self.clone())
    }
}

impl FromStr for VirtualNetworkPeeringId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        VirtualNetworkPeeringId::try_from_expanded(s)
    }
}

impl Serialize for VirtualNetworkPeeringId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for VirtualNetworkPeeringId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = VirtualNetworkPeeringId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::VirtualNetworkPeeringId;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let mut raw = [0u8; 64];
        rand::rng().fill(&mut raw);
        let mut un = Unstructured::new(&raw);
        let name = VirtualNetworkPeeringId::arbitrary(&mut un)?;
        let serialized = serde_json::to_string(&name)?;
        let deserialized: VirtualNetworkPeeringId = serde_json::from_str(&serialized)?;
        assert_eq!(name, deserialized);
        Ok(())
    }
}
