use crate::AzureContainerInstanceResourceName;
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

pub const AZURE_CONTAINER_INSTANCE_RESOURCE_ID_PREFIX: &str =
    "/providers/Microsoft.ContainerInstance/containerGroups/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureContainerInstanceResourceId {
    pub resource_group_id: ResourceGroupId,
    pub azure_container_instance_resource_name: AzureContainerInstanceResourceName,
}
crate::impl_facet_string_proxy!(AzureContainerInstanceResourceId, value => value.expanded_form());

impl AzureContainerInstanceResourceId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        azure_container_instance_resource_name: impl Into<AzureContainerInstanceResourceName>,
    ) -> Self {
        Self {
            resource_group_id: resource_group_id.into(),
            azure_container_instance_resource_name: azure_container_instance_resource_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureContainerInstanceResourceName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let name = name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert container instance resource name")?;
        Ok(Self::new(resource_group_id, name))
    }
}

impl HasSlug for AzureContainerInstanceResourceId {
    type Name = AzureContainerInstanceResourceName;

    fn name(&self) -> &Self::Name {
        &self.azure_container_instance_resource_name
    }
}

impl AsRef<ResourceGroupId> for AzureContainerInstanceResourceId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<AzureContainerInstanceResourceName> for AzureContainerInstanceResourceId {
    fn as_ref(&self) -> &AzureContainerInstanceResourceName {
        &self.azure_container_instance_resource_name
    }
}

impl NameValidatable for AzureContainerInstanceResourceId {
    fn validate_name(name: &str) -> Result<()> {
        AzureContainerInstanceResourceName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for AzureContainerInstanceResourceId {
    fn get_prefix() -> &'static str {
        AZURE_CONTAINER_INSTANCE_RESOURCE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for AzureContainerInstanceResourceId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self::new(resource_group_id, name)
    }
}

impl FromStr for AzureContainerInstanceResourceId {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from_expanded(value)
    }
}

impl Scope for AzureContainerInstanceResourceId {
    type Err = <Self as FromStr>::Err;

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        Self::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            AZURE_CONTAINER_INSTANCE_RESOURCE_ID_PREFIX,
            self.azure_container_instance_resource_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ContainerInstanceResource
    }

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::AzureContainerInstanceResource(self.clone())
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerInstanceResourceId);
cloud_terrastodon_registry::register_arbitrary!(AzureContainerInstanceResourceId);
