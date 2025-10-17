use crate::prelude::ResourceGroupId;
use crate::prelude::VirtualNetworkName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use crate::slug::HasSlug;
use crate::slug::Slug;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

pub const VIRTUAL_NETWORK_ID_PREFIX: &str = "/providers/Microsoft.Network/virtualNetworks/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct VirtualNetworkId {
    pub resource_group_id: ResourceGroupId,
    pub virtual_network_name: VirtualNetworkName,
}

impl VirtualNetworkId {
    pub fn new(
        resource_group_id: ResourceGroupId,
        virtual_network_name: impl Into<VirtualNetworkName>,
    ) -> Self {
        Self {
            resource_group_id,
            virtual_network_name: virtual_network_name.into(),
        }
    }

    pub fn try_new<N>(resource_group_id: ResourceGroupId, virtual_network_name: N) -> Result<Self>
    where
        N: TryInto<VirtualNetworkName>,
        N::Error: Into<eyre::Error>,
    {
        Ok(Self {
            resource_group_id,
            virtual_network_name: virtual_network_name
                .try_into()
                .map_err(Into::into)
                .context("Failed to convert to VirtualNetworkName")?,
        })
    }
}

impl HasSlug for VirtualNetworkId {
    type Name = VirtualNetworkName;

    fn name(&self) -> &Self::Name {
        &self.virtual_network_name
    }
}

impl AsRef<ResourceGroupId> for VirtualNetworkId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<VirtualNetworkName> for VirtualNetworkId {
    fn as_ref(&self) -> &VirtualNetworkName {
        &self.virtual_network_name
    }
}

impl NameValidatable for VirtualNetworkId {
    fn validate_name(name: &str) -> Result<()> {
        VirtualNetworkName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for VirtualNetworkId {
    fn get_prefix() -> &'static str {
        VIRTUAL_NETWORK_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for VirtualNetworkId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_group_id,
            virtual_network_name: name,
        }
    }
}

impl Scope for VirtualNetworkId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        VirtualNetworkId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            VIRTUAL_NETWORK_ID_PREFIX,
            self.virtual_network_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::VirtualNetwork
    }

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        crate::scopes::ScopeImpl::VirtualNetwork(self.clone())
    }
}

impl FromStr for VirtualNetworkId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        VirtualNetworkId::try_from_expanded(s)
    }
}

impl Serialize for VirtualNetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.expanded_form().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VirtualNetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from_expanded(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::SubscriptionId;
    use std::str::FromStr;

    #[test]
    fn test_virtual_network_id_serialization_deserialization() -> eyre::Result<()> {
        let sub_id = SubscriptionId::from_str("00000000-0000-0000-0000-000000000000")?;
        let rg_id = ResourceGroupId::new(sub_id, ResourceGroupName::try_new("test-rg").unwrap());
        let vnet_id = VirtualNetworkId::try_new(rg_id, "test-vnet")?;

        let serialized = serde_json::to_string(&vnet_id)?;
        let expected_str = "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/test-rg/providers/Microsoft.Network/virtualNetworks/test-vnet".to_string();
        assert_eq!(serialized, serde_json::to_string(&expected_str)?);

        let deserialized: VirtualNetworkId = serde_json::from_str(&serialized)?;
        assert_eq!(vnet_id, deserialized);

        Ok(())
    }

    #[test]
    fn test_scope_roundtrip() -> eyre::Result<()> {
        let sub_id = SubscriptionId::from_str("11111111-1111-1111-1111-111111111111")?;
        let rg_id = ResourceGroupId::new(
            sub_id,
            ResourceGroupName::try_new("myResourceGroup").unwrap(),
        );
        let original_id = VirtualNetworkId::try_new(rg_id, "myVirtualNetwork")?;
        let expanded = original_id.expanded_form();
        let parsed_id = VirtualNetworkId::try_from_expanded(&expanded)?;
        assert_eq!(original_id, parsed_id);
        Ok(())
    }

    #[test]
    fn test_name_validation() -> eyre::Result<()> {
        assert!(VirtualNetworkId::validate_name("valid-vnet-name").is_ok());
        assert!(VirtualNetworkId::validate_name("a").is_err()); // Too short
        Ok(())
    }
}
