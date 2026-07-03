use crate::AzureCognitiveServicesAccountResourceName;
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
use std::str::FromStr;

pub const AZURE_COGNITIVE_SERVICES_ACCOUNT_RESOURCE_ID_PREFIX: &str =
    "/providers/Microsoft.CognitiveServices/accounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureCognitiveServicesAccountResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_cognitive_services_account_resource_name: AzureCognitiveServicesAccountResourceName,
}
crate::impl_facet_string_proxy!(AzureCognitiveServicesAccountResourceId, value => value.expanded_form());

impl AzureCognitiveServicesAccountResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_cognitive_services_account_resource_name: impl Into<
            AzureCognitiveServicesAccountResourceName,
        >,
    ) -> Self {
        Self {
            resource_group_id: resource_group_id.into(),
            azure_cognitive_services_account_resource_name:
                azure_cognitive_services_account_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(
        resource_group_id: R,
        azure_cognitive_services_account_resource_name: N,
    ) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureCognitiveServicesAccountResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let azure_cognitive_services_account_resource_name =
            azure_cognitive_services_account_resource_name
                .try_into()
                .map_err(Into::into)
                .wrap_err("Failed to convert azure_cognitive_services_account_resource_name")?;
        Ok(Self {
            resource_group_id,
            azure_cognitive_services_account_resource_name,
        })
    }
}

impl HasSlug for AzureCognitiveServicesAccountResourceId {
    type Name = AzureCognitiveServicesAccountResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_cognitive_services_account_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzureCognitiveServicesAccountResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzureCognitiveServicesAccountResourceName> for AzureCognitiveServicesAccountResourceId {
    fn as_ref(&self) -> &AzureCognitiveServicesAccountResourceName {
        &self.azure_cognitive_services_account_resource_name
    }
}

impl NameValidatable for AzureCognitiveServicesAccountResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzureCognitiveServicesAccountResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzureCognitiveServicesAccountResourceId {
    fn get_prefix() -> &'static str {
        AZURE_COGNITIVE_SERVICES_ACCOUNT_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzureCognitiveServicesAccountResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_group_id,
            azure_cognitive_services_account_resource_name: name,
        }
    }
}

impl FromStr for AzureCognitiveServicesAccountResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::try_from_expanded(s)
    }
}

impl Scope for AzureCognitiveServicesAccountResourceId {
    type Err = <Self as FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        Self::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_COGNITIVE_SERVICES_ACCOUNT_RESOURCE_ID_PREFIX,
            self.azure_cognitive_services_account_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::CognitiveServicesAccountResource
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::AzureCognitiveServicesAccountResource(self.clone())
    }
}

cloud_terrastodon_registry::register_thing!(AzureCognitiveServicesAccountResourceId);

#[cfg(test)]
mod test {
    use super::AzureCognitiveServicesAccountResourceId;
    use crate::AzureCognitiveServicesAccountResourceName;
    use crate::ResourceGroupId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn it_works() -> eyre::Result<()> {
        assert!(AzureCognitiveServicesAccountResourceId::try_new("", "").is_err());
        AzureCognitiveServicesAccountResourceId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "my-openai",
        )?;
        AzureCognitiveServicesAccountResourceId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "my-openai",
        )?;
        AzureCognitiveServicesAccountResourceId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "my-openai",
        )?;
        AzureCognitiveServicesAccountResourceId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzureCognitiveServicesAccountResourceName::try_new("my-openai")?,
        );
        Ok(())
    }

    #[test]
    fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzureCognitiveServicesAccountResourceId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzureCognitiveServicesAccountResourceId = serialized.parse()?;
            assert_eq!(id, deserialized);
            let json = facet_json::to_string(&serialized)?;
            assert_eq!(facet_json::to_string(&id)?, json);
            assert_eq!(
                facet_json::from_str::<AzureCognitiveServicesAccountResourceId>(&json)?,
                id
            );
        }
        Ok(())
    }
}
