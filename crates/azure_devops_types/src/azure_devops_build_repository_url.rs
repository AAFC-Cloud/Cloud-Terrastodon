use arbitrary::Arbitrary;
use eyre::Result;
use std::str::FromStr;
use url::Url;

/// The source-provider URL of a build repository.
///
/// Microsoft documentation: [BuildRepository.url](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildrepository).
/// Repositories can belong to different source providers, so no common Azure
/// organization/project route, scheme or host restriction is documented.
/// Preserve a parsed URL rather than imposing one provider's resource identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildRepositoryUrl(pub Url);

impl AzureDevOpsBuildRepositoryUrl {
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

impl FromStr for AzureDevOpsBuildRepositoryUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildRepositoryUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<Url> for AzureDevOpsBuildRepositoryUrl {
    fn from(value: Url) -> Self {
        Self(value)
    }
}

impl From<&AzureDevOpsBuildRepositoryUrl> for String {
    fn from(value: &AzureDevOpsBuildRepositoryUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl AsRef<Url> for AzureDevOpsBuildRepositoryUrl {
    fn as_ref(&self) -> &Url {
        self.as_url()
    }
}

impl AsRef<str> for AzureDevOpsBuildRepositoryUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsBuildRepositoryUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildRepositoryUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let identifier = u64::arbitrary(u)?;
        format!("https://repositories.invalid/synthetic/{identifier}")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRepositoryUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRepositoryUrl);
