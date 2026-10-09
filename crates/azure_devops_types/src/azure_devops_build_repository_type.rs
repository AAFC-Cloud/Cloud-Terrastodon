use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The source-provider identifier of a build repository.
///
/// Microsoft documentation: [BuildRepository](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildrepository).
/// This library rejects blank values and control characters as local safety
/// constraints, without imposing undocumented provider naming restrictions.
/// Exact spelling, spaces, Unicode and provider-specific separators are preserved.
/// Provider identifiers remain open strings so new source providers are preserved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildRepositoryType(String);

impl AzureDevOpsBuildRepositoryType {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "The source-provider identifier of a build repository must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "The source-provider identifier of a build repository must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildRepositoryType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildRepositoryType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildRepositoryType {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildRepositoryType {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildRepositoryType> for String {
    fn from(value: &AzureDevOpsBuildRepositoryType) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildRepositoryType> for String {
    fn from(value: AzureDevOpsBuildRepositoryType) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildRepositoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildRepositoryType {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?)
            .unwrap_or_else(|_| Self::try_new("SyntheticProvider").expect("valid synthetic value")))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRepositoryType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRepositoryType);
