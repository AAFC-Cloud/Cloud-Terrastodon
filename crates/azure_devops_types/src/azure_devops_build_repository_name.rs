use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The friendly name of a repository used by a build definition.
///
/// Microsoft documentation: [BuildRepository](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildrepository).
/// This library rejects blank values and control characters as local safety
/// constraints, without imposing undocumented provider naming restrictions.
/// Exact spelling, spaces, Unicode and provider-specific separators are preserved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildRepositoryName(String);

impl AzureDevOpsBuildRepositoryName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "The friendly name of a repository used by a build definition must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "The friendly name of a repository used by a build definition must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildRepositoryName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildRepositoryName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildRepositoryName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildRepositoryName {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildRepositoryName> for String {
    fn from(value: &AzureDevOpsBuildRepositoryName) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildRepositoryName> for String {
    fn from(value: AzureDevOpsBuildRepositoryName) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildRepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildRepositoryName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?).unwrap_or_else(|_| {
            Self::try_new("Synthetic repository").expect("valid synthetic value")
        }))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRepositoryName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRepositoryName);
