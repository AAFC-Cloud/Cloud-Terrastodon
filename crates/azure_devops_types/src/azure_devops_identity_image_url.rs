use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use url::Url;

/// The image URL returned for an Azure DevOps identity.
///
/// See `imageUrl` in Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// This deprecated property is also available through the identity's `avatar`
/// reference link. No universal image host or route is specified, so preserve
/// the parsed URL without imposing additional scheme, host, path, query, or
/// fragment restrictions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsIdentityImageUrl(Url);

impl AzureDevOpsIdentityImageUrl {
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

impl FromStr for AzureDevOpsIdentityImageUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsIdentityImageUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureDevOpsIdentityImageUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<Url> for AzureDevOpsIdentityImageUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl AsRef<str> for AzureDevOpsIdentityImageUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Url> for AzureDevOpsIdentityImageUrl {
    fn as_ref(&self) -> &Url {
        self.as_url()
    }
}

impl From<&AzureDevOpsIdentityImageUrl> for String {
    fn from(value: &AzureDevOpsIdentityImageUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl From<AzureDevOpsIdentityImageUrl> for Url {
    fn from(value: AzureDevOpsIdentityImageUrl) -> Self {
        value.into_url()
    }
}

impl std::fmt::Display for AzureDevOpsIdentityImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsIdentityImageUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identifier = u64::arbitrary(u)?;
        format!("https://images.invalid/identity/{identifier}")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityImageUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityImageUrl);
