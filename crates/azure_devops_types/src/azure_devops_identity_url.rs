use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use url::Url;

/// The source-resource URL returned for an Azure DevOps identity.
///
/// See `url` in Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// The schema describes a source-resource URL without a universal host or route
/// contract. Its scope may belong to a distributed identity service rather than
/// an organization URL. Preserve the parsed URL without inferring an organization
/// or imposing additional scheme, host, path, query, or fragment restrictions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsIdentityUrl(Url);

impl AzureDevOpsIdentityUrl {
    /// Parse the service-provided URL using `url::Url`'s URL semantics.
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        Ok(Self(Url::parse(value.as_ref())?))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

impl FromStr for AzureDevOpsIdentityUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsIdentityUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureDevOpsIdentityUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<Url> for AzureDevOpsIdentityUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl AsRef<str> for AzureDevOpsIdentityUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Url> for AzureDevOpsIdentityUrl {
    fn as_ref(&self) -> &Url {
        self.as_url()
    }
}

impl From<&AzureDevOpsIdentityUrl> for String {
    fn from(value: &AzureDevOpsIdentityUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl From<AzureDevOpsIdentityUrl> for Url {
    fn from(value: AzureDevOpsIdentityUrl) -> Self {
        value.into_url()
    }
}

impl std::fmt::Display for AzureDevOpsIdentityUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsIdentityUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identifier = u64::arbitrary(u)?;
        format!("https://identities.invalid/source/{identifier}")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityUrl);
