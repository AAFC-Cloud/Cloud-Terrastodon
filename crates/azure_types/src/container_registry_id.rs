use crate::prelude::ContainerRegistryName;
use crate::prelude::ResourceGroupId;
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

pub const CONTAINER_REGISTRY_ID_PREFIX: &str = "/providers/Microsoft.ContainerRegistry/registries/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct ContainerRegistryId {
    pub resource_group_id: ResourceGroupId,
    pub container_registry_name: ContainerRegistryName,
}
impl ContainerRegistryId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        container_registry_name: impl Into<ContainerRegistryName>,
    ) -> ContainerRegistryId {
        ContainerRegistryId {
            resource_group_id: resource_group_id.into(),
            container_registry_name: container_registry_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, container_registry_name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<ContainerRegistryName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let container_registry_name = container_registry_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert container_registry_name")?;
        Ok(ContainerRegistryId {
            resource_group_id,
            container_registry_name,
        })
    }
}

impl HasSlug for ContainerRegistryId {
    type Name = ContainerRegistryName;

    fn name(&self) -> &Self::Name {
        &self.container_registry_name
    }
}
impl AsRef<ResourceGroupId> for ContainerRegistryId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl AsRef<ContainerRegistryName> for ContainerRegistryId {
    fn as_ref(&self) -> &ContainerRegistryName {
        &self.container_registry_name
    }
}

impl NameValidatable for ContainerRegistryId {
    fn validate_name(name: &str) -> Result<()> {
        ContainerRegistryName::try_new(name).map(|_| ())
    }
}
impl HasPrefix for ContainerRegistryId {
    fn get_prefix() -> &'static str {
        CONTAINER_REGISTRY_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for ContainerRegistryId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        ContainerRegistryId {
            resource_group_id,
            container_registry_name: name,
        }
    }
}

impl Scope for ContainerRegistryId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ContainerRegistryId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            CONTAINER_REGISTRY_ID_PREFIX,
            self.container_registry_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ContainerRegistry
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ContainerRegistry(self.clone())
    }
}

impl FromStr for ContainerRegistryId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ContainerRegistryId::try_from_expanded(s)
    }
}

impl Serialize for ContainerRegistryId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for ContainerRegistryId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = ContainerRegistryId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::ContainerRegistryId;
    use crate::prelude::ContainerRegistryName;
    use crate::prelude::ResourceGroupId;
    use crate::prelude::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(ContainerRegistryId::try_new("", "").is_err());
        ContainerRegistryId::try_from_expanded(
            "/subscriptions/6998ecdd-44d1-43f9-bb47-49e77cbd5d28/resourceGroups/myRg/providers/Microsoft.ContainerRegistry/registries/myconTainerRegisTry",
        )?;
        ContainerRegistryId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "aaaaa",
        )?;
        ContainerRegistryId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "aaaaa",
        )?;
        ContainerRegistryId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "aaaaa",
        )?;
        ContainerRegistryId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            ContainerRegistryName::try_new("aaaaa")?,
        );
        Ok(())
    }
}
