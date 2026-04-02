use crate::AzureAppServiceResourceName;
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

pub const AZURE_APP_SERVICE_RESOURCE_ID_PREFIX: &str = "/providers/Microsoft.Web/sites/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AzureAppServiceResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_app_service_resource_name: AzureAppServiceResourceName,
}

impl AzureAppServiceResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_app_service_resource_name: impl Into<AzureAppServiceResourceName>,
    ) -> AzureAppServiceResourceId {
        AzureAppServiceResourceId {
            resource_group_id: resource_group_id.into(),
            azure_app_service_resource_name: azure_app_service_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, azure_app_service_resource_name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureAppServiceResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let azure_app_service_resource_name = azure_app_service_resource_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert azure_app_service_resource_name")?;
        Ok(AzureAppServiceResourceId {
            resource_group_id,
            azure_app_service_resource_name,
        })
    }
}

impl HasSlug for AzureAppServiceResourceId {
    type Name = AzureAppServiceResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_app_service_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzureAppServiceResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzureAppServiceResourceName> for AzureAppServiceResourceId {
    fn as_ref(&self) -> &AzureAppServiceResourceName {
        &self.azure_app_service_resource_name
    }
}

impl NameValidatable for AzureAppServiceResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzureAppServiceResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzureAppServiceResourceId {
    fn get_prefix() -> &'static str {
        AZURE_APP_SERVICE_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzureAppServiceResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        AzureAppServiceResourceId {
            resource_group_id,
            azure_app_service_resource_name: name,
        }
    }
}

impl FromStr for AzureAppServiceResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        AzureAppServiceResourceId::try_from_expanded(s)
    }
}

impl Scope for AzureAppServiceResourceId {
    type Err = <Self as std::str::FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        AzureAppServiceResourceId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_APP_SERVICE_RESOURCE_ID_PREFIX,
            self.azure_app_service_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::AppServiceResource
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::AzureAppServiceResource(self.clone())
    }
}

impl Serialize for AzureAppServiceResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzureAppServiceResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = AzureAppServiceResourceId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::AzureAppServiceResourceId;
    use crate::AzureAppServiceResourceName;
    use crate::ResourceGroupId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn it_works() -> eyre::Result<()> {
        assert!(AzureAppServiceResourceId::try_new("", "").is_err());
        AzureAppServiceResourceId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "my-app-service",
        )?;
        AzureAppServiceResourceId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "my-app-service",
        )?;
        AzureAppServiceResourceId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "my-app-service",
        )?;
        AzureAppServiceResourceId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzureAppServiceResourceName::try_new("my-app-service")?,
        );
        Ok(())
    }

    #[test]
    fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzureAppServiceResourceId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzureAppServiceResourceId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
