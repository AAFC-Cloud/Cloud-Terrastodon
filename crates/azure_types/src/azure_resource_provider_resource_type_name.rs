use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// A provider-relative resource-type path, such as `virtualMachines/extensions`.
///
/// This is the `resourceType` property of the
/// [Resource Management 2021-04-01 provider resource-type schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#providerresourcetype),
/// not a namespace-qualified `ResourceType`. Nonempty path segments are a
/// library composition invariant; no character allowlist or length limit is added.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Facet)]
#[facet(proxy = String)]
pub struct AzureResourceProviderResourceTypeName {
    // Private so mutation cannot introduce an absolute or empty-segment path.
    value: String,
}

impl AzureResourceProviderResourceTypeName {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.split('/').any(str::is_empty),
            "provider-relative resource type must have nonempty path segments: {value:?}"
        );
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl Display for AzureResourceProviderResourceTypeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for AzureResourceProviderResourceTypeName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureResourceProviderResourceTypeName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

crate::impl_facet_string_proxy!(AzureResourceProviderResourceTypeName, value => value.to_string());

impl<'a> Arbitrary<'a> for AzureResourceProviderResourceTypeName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let segments = u.int_in_range(1..=4)?;
        let mut names = Vec::with_capacity(segments);
        for _ in 0..segments {
            names.push(format!("resources{}", u.arbitrary::<u32>()?));
        }
        Self::new(names.join("/")).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderResourceTypeName);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderResourceTypeName);

#[cfg(test)]
mod tests {
    use super::AzureResourceProviderResourceTypeName;

    #[test]
    fn rejects_empty_segments_without_normalizing_service_names() -> eyre::Result<()> {
        for invalid in ["", "/widgets", "widgets/", "widgets//children"] {
            assert!(AzureResourceProviderResourceTypeName::new(invalid).is_err());
        }
        let name = AzureResourceProviderResourceTypeName::new("widgets/childTypes")?;
        assert_eq!(name.as_str(), "widgets/childTypes");
        Ok(())
    }
}
