use crate::AzureResourceProviderResourceTypeName;
use crate::ResourceType;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// The namespace in a resource provider's `namespace` property.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#provider).
/// The namespace remains open to newly advertised providers. Requiring one
/// nonempty path segment is a library composition invariant, not an additional
/// Azure naming policy; spelling and case are preserved.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Facet)]
#[facet(proxy = String)]
pub struct AzureResourceProviderNamespace {
    // Private so mutation cannot introduce an empty or nested namespace.
    value: String,
}

impl AzureResourceProviderNamespace {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.is_empty() && !value.contains('/'),
            "resource provider namespace must be one nonempty path segment: {value:?}"
        );
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Qualify a provider-relative resource type using the generated vocabulary.
    /// Unknown types remain `ResourceType::Other` in every build, allowing a
    /// catalogue refresh to discover types absent from the current generated file.
    /// Known variants use that file's canonical spelling; use the two component
    /// values directly when preserving the service's exact spelling matters.
    pub fn resource_type(&self, name: &AzureResourceProviderResourceTypeName) -> ResourceType {
        ResourceType::from(format!("{self}/{name}"))
    }
}

impl Display for AzureResourceProviderNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for AzureResourceProviderNamespace {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureResourceProviderNamespace {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

crate::impl_facet_string_proxy!(AzureResourceProviderNamespace, value => value.to_string());

impl<'a> Arbitrary<'a> for AzureResourceProviderNamespace {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(format!("Example.Provider{}", u.arbitrary::<u32>()?))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderNamespace);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderNamespace);

#[cfg(test)]
mod tests {
    use super::AzureResourceProviderNamespace;
    use crate::AzureResourceProviderResourceTypeName;
    use crate::ResourceType;

    #[test]
    fn qualification_preserves_unknown_nested_types_and_resolves_known_types() -> eyre::Result<()> {
        let namespace = AzureResourceProviderNamespace::new("Example.FutureProvider")?;
        let name = AzureResourceProviderResourceTypeName::new("widgets/children")?;
        assert_eq!(
            namespace.resource_type(&name),
            ResourceType::Other("Example.FutureProvider/widgets/children".to_owned())
        );

        let namespace = AzureResourceProviderNamespace::new("microsoft.compute")?;
        let name = AzureResourceProviderResourceTypeName::new("virtualMachines")?;
        assert_eq!(
            namespace.resource_type(&name),
            ResourceType::MICROSOFT_DOT_COMPUTE_SLASH_VIRTUALMACHINES
        );
        assert_eq!(namespace.as_str(), "microsoft.compute");
        assert_eq!(name.as_str(), "virtualMachines");
        Ok(())
    }

    #[test]
    fn namespace_cannot_replace_the_relative_type_path() {
        assert!(AzureResourceProviderNamespace::new("").is_err());
        assert!(AzureResourceProviderNamespace::new("Example.Provider/widgets").is_err());
    }
}
