use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;

// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcontainerregistry
// https://docs.docker.com/docker-hub/repos/create/
// https://docs.docker.com/get-started/docker-concepts/building-images/build-tag-and-publish-an-image/#tagging-images
// https://learn.microsoft.com/en-us/azure/container-registry/container-registry-concepts
// https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/container_registry
// https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry
// https://kubernetes.io/docs/concepts/containers/images/
// https://github.com/moby/moby/blob/be97c66708c24727836a22247319ff2943d91a03/daemon/names/names.go
/// I was unable to find a definitive source for the rules governing container registry repository names.
///
/// For now, this type will always successfully validate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct ContainerRegistryRepositoryName {
    inner: CompactString,
}
impl Slug for ContainerRegistryRepositoryName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        Ok(())
    }
}

impl FromStr for ContainerRegistryRepositoryName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContainerRegistryRepositoryName::try_new(s)
    }
}
impl TryFrom<&str> for ContainerRegistryRepositoryName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerRegistryRepositoryName::try_new(value)
    }
}
impl std::fmt::Display for ContainerRegistryRepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for ContainerRegistryRepositoryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ContainerRegistryRepositoryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}
impl Deref for ContainerRegistryRepositoryName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl TryFrom<CompactString> for ContainerRegistryRepositoryName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ContainerRegistryRepositoryName> for CompactString {
    fn from(value: ContainerRegistryRepositoryName) -> Self {
        value.inner
    }
}
#[cfg(test)]
mod test {
    use crate::prelude::ContainerRegistryRepositoryName;
    use crate::slug::Slug;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(ContainerRegistryRepositoryName::try_new("bruh").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("-").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a-b-c").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("hi+hi").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("aa").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("JEOFF").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("caPital").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("aaaa").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(23)).is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(24)).is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(25)).is_ok());
        Ok(())
    }
}
