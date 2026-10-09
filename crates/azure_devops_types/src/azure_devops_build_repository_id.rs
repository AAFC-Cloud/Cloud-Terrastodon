use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The provider-specific identifier of a build repository.
///
/// Microsoft documentation: [BuildRepository](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildrepository).
/// This library rejects blank values and control characters as local safety
/// constraints, without imposing undocumented provider naming restrictions.
/// Exact spelling, spaces, Unicode and provider-specific separators are preserved.
/// The build API documents a string, unlike Azure Repos' UUID-only repository ID.
/// GitHub owner/name identities and other source-provider identifiers remain valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildRepositoryId(String);

impl AzureDevOpsBuildRepositoryId {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "The provider-specific identifier of a build repository must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "The provider-specific identifier of a build repository must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildRepositoryId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildRepositoryId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildRepositoryId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildRepositoryId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildRepositoryId> for String {
    fn from(value: &AzureDevOpsBuildRepositoryId) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildRepositoryId> for String {
    fn from(value: AzureDevOpsBuildRepositoryId) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildRepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildRepositoryId {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?).unwrap_or_else(|_| {
            Self::try_new("synthetic-owner/synthetic-repository").expect("valid synthetic value")
        }))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRepositoryId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRepositoryId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_ids_preserve_provider_identity_instead_of_requiring_uuid() {
        for identity in [
            "12345678-1234-1234-1234-123456789012",
            "synthetic-owner/synthetic-repository",
            "$/Synthetic project",
        ] {
            let value = AzureDevOpsBuildRepositoryId::try_new(identity).unwrap();
            assert_eq!(value.as_str(), identity);
        }
        for invalid in ["", "  ", "repository\u{1b}"] {
            assert!(AzureDevOpsBuildRepositoryId::try_new(invalid).is_err());
        }
    }
}
