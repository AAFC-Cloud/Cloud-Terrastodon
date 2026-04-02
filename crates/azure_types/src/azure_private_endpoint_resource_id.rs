use crate::AzurePrivateEndpointResourceName;
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

pub const AZURE_PRIVATE_ENDPOINT_RESOURCE_ID_PREFIX: &str =
    "/providers/Microsoft.Network/privateEndpoints/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AzurePrivateEndpointResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_private_endpoint_resource_name: AzurePrivateEndpointResourceName,
}

impl AzurePrivateEndpointResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_private_endpoint_resource_name: impl Into<AzurePrivateEndpointResourceName>,
    ) -> AzurePrivateEndpointResourceId {
        AzurePrivateEndpointResourceId {
            resource_group_id: resource_group_id.into(),
            azure_private_endpoint_resource_name: azure_private_endpoint_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(
        resource_group_id: R,
        azure_private_endpoint_resource_name: N,
    ) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzurePrivateEndpointResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let azure_private_endpoint_resource_name = azure_private_endpoint_resource_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert azure_private_endpoint_resource_name")?;
        Ok(AzurePrivateEndpointResourceId {
            resource_group_id,
            azure_private_endpoint_resource_name,
        })
    }
}

impl HasSlug for AzurePrivateEndpointResourceId {
    type Name = AzurePrivateEndpointResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_private_endpoint_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzurePrivateEndpointResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzurePrivateEndpointResourceName> for AzurePrivateEndpointResourceId {
    fn as_ref(&self) -> &AzurePrivateEndpointResourceName {
        &self.azure_private_endpoint_resource_name
    }
}

impl NameValidatable for AzurePrivateEndpointResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzurePrivateEndpointResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzurePrivateEndpointResourceId {
    fn get_prefix() -> &'static str {
        AZURE_PRIVATE_ENDPOINT_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzurePrivateEndpointResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        AzurePrivateEndpointResourceId {
            resource_group_id,
            azure_private_endpoint_resource_name: name,
        }
    }
}

impl FromStr for AzurePrivateEndpointResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        AzurePrivateEndpointResourceId::try_from_expanded(s)
    }
}

impl Scope for AzurePrivateEndpointResourceId {
    type Err = <Self as std::str::FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        AzurePrivateEndpointResourceId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_PRIVATE_ENDPOINT_RESOURCE_ID_PREFIX,
            self.azure_private_endpoint_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PrivateEndpointResource
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::AzurePrivateEndpointResource(self.clone())
    }
}

impl Serialize for AzurePrivateEndpointResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzurePrivateEndpointResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = AzurePrivateEndpointResourceId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::AzurePrivateEndpointResourceId;
    use crate::AzurePrivateEndpointResourceName;
    use crate::ResourceGroupId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(AzurePrivateEndpointResourceId::try_new("", "").is_err());
        AzurePrivateEndpointResourceId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "pe01",
        )?;
        AzurePrivateEndpointResourceId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "pe01",
        )?;
        AzurePrivateEndpointResourceId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "pe01",
        )?;
        AzurePrivateEndpointResourceId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzurePrivateEndpointResourceName::try_new("pe01")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzurePrivateEndpointResourceId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzurePrivateEndpointResourceId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
