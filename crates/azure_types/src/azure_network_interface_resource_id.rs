use crate::AzureNetworkInterfaceResourceName;
use crate::ResourceGroupId;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
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

pub const AZURE_NETWORK_INTERFACE_RESOURCE_ID_PREFIX: &str =
    "/providers/Microsoft.Network/networkInterfaces/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AzureNetworkInterfaceResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_network_interface_resource_name: AzureNetworkInterfaceResourceName,
}

impl AzureNetworkInterfaceResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_network_interface_resource_name: impl Into<AzureNetworkInterfaceResourceName>,
    ) -> AzureNetworkInterfaceResourceId {
        AzureNetworkInterfaceResourceId {
            resource_group_id: resource_group_id.into(),
            azure_network_interface_resource_name: azure_network_interface_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(
        resource_group_id: R,
        azure_network_interface_resource_name: N,
    ) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureNetworkInterfaceResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let azure_network_interface_resource_name = azure_network_interface_resource_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert azure_network_interface_resource_name")?;
        Ok(AzureNetworkInterfaceResourceId {
            resource_group_id,
            azure_network_interface_resource_name,
        })
    }
}

impl HasSlug for AzureNetworkInterfaceResourceId {
    type Name = AzureNetworkInterfaceResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_network_interface_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzureNetworkInterfaceResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzureNetworkInterfaceResourceName> for AzureNetworkInterfaceResourceId {
    fn as_ref(&self) -> &AzureNetworkInterfaceResourceName {
        &self.azure_network_interface_resource_name
    }
}

impl NameValidatable for AzureNetworkInterfaceResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzureNetworkInterfaceResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzureNetworkInterfaceResourceId {
    fn get_prefix() -> &'static str {
        AZURE_NETWORK_INTERFACE_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzureNetworkInterfaceResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        AzureNetworkInterfaceResourceId {
            resource_group_id,
            azure_network_interface_resource_name: name,
        }
    }
}

impl FromStr for AzureNetworkInterfaceResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        AzureNetworkInterfaceResourceId::try_from_expanded(s)
    }
}

impl Scope for AzureNetworkInterfaceResourceId {
    type Err = <Self as std::str::FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        AzureNetworkInterfaceResourceId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_NETWORK_INTERFACE_RESOURCE_ID_PREFIX,
            self.azure_network_interface_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::NetworkInterfaceResource
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::AzureNetworkInterfaceResource(self.clone())
    }
}

impl Serialize for AzureNetworkInterfaceResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzureNetworkInterfaceResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = AzureNetworkInterfaceResourceId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::AzureNetworkInterfaceResourceId;
    use crate::AzureNetworkInterfaceResourceName;
    use crate::ResourceGroupId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(AzureNetworkInterfaceResourceId::try_new("", "").is_err());
        AzureNetworkInterfaceResourceId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "nic01",
        )?;
        AzureNetworkInterfaceResourceId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "nic01",
        )?;
        AzureNetworkInterfaceResourceId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "nic01",
        )?;
        AzureNetworkInterfaceResourceId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzureNetworkInterfaceResourceName::try_new("nic01")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzureNetworkInterfaceResourceId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzureNetworkInterfaceResourceId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
