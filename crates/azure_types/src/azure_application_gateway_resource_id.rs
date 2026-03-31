use crate::AzureApplicationGatewayResourceName;
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

pub const AZURE_APPLICATION_GATEWAY_RESOURCE_ID_PREFIX: &str =
    "/providers/Microsoft.Network/applicationGateways/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AzureApplicationGatewayResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_application_gateway_resource_name: AzureApplicationGatewayResourceName,
}

impl AzureApplicationGatewayResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_application_gateway_resource_name: impl Into<AzureApplicationGatewayResourceName>,
    ) -> Self {
        Self {
            resource_group_id: resource_group_id.into(),
            azure_application_gateway_resource_name: azure_application_gateway_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(
        resource_group_id: R,
        azure_application_gateway_resource_name: N,
    ) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureApplicationGatewayResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let azure_application_gateway_resource_name = azure_application_gateway_resource_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert azure_application_gateway_resource_name")?;
        Ok(Self {
            resource_group_id,
            azure_application_gateway_resource_name,
        })
    }
}

impl HasSlug for AzureApplicationGatewayResourceId {
    type Name = AzureApplicationGatewayResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_application_gateway_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzureApplicationGatewayResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzureApplicationGatewayResourceName> for AzureApplicationGatewayResourceId {
    fn as_ref(&self) -> &AzureApplicationGatewayResourceName {
        &self.azure_application_gateway_resource_name
    }
}

impl NameValidatable for AzureApplicationGatewayResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzureApplicationGatewayResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzureApplicationGatewayResourceId {
    fn get_prefix() -> &'static str {
        AZURE_APPLICATION_GATEWAY_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzureApplicationGatewayResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_group_id,
            azure_application_gateway_resource_name: name,
        }
    }
}

impl FromStr for AzureApplicationGatewayResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::try_from_expanded(s)
    }
}

impl Scope for AzureApplicationGatewayResourceId {
    type Err = <Self as std::str::FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        Self::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_APPLICATION_GATEWAY_RESOURCE_ID_PREFIX,
            self.azure_application_gateway_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ApplicationGatewayResource
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::AzureApplicationGatewayResource(self.clone())
    }
}

impl Serialize for AzureApplicationGatewayResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzureApplicationGatewayResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = Self::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::AzureApplicationGatewayResourceId;
    use crate::AzureApplicationGatewayResourceName;
    use crate::ResourceGroupId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(AzureApplicationGatewayResourceId::try_new("", "").is_err());
        AzureApplicationGatewayResourceId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "my-app-gateway",
        )?;
        AzureApplicationGatewayResourceId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "my-app-gateway",
        )?;
        AzureApplicationGatewayResourceId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "my-app-gateway",
        )?;
        AzureApplicationGatewayResourceId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzureApplicationGatewayResourceName::try_new("my-app-gateway")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzureApplicationGatewayResourceId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzureApplicationGatewayResourceId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
