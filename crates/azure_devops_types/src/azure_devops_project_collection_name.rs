use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The exact name of an existing Azure DevOps Server project collection.
///
/// Follows Microsoft's documented [collection name spelling and length rules](https://learn.microsoft.com/en-us/azure/devops/organizations/settings/naming-restrictions?view=azure-devops#project-collection-names).
/// Spaces and Unicode are valid; the length limit is 64 UTF-16 code units, as
/// described in the page's general considerations. Rust strings cannot contain
/// lone surrogate code points. Blank names are rejected as a library constraint.
///
/// This value identifies an existing collection rather than deciding whether a
/// new collection may be created. Creation must separately check uniqueness and
/// reserved names. In particular, the documented default `DefaultCollection`
/// remains valid here despite its inclusion in the general reserved-name list.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsProjectCollectionName(CompactString);

impl AzureDevOpsProjectCollectionName {
    pub fn try_new(value: impl Into<CompactString>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "Project collection names must not be blank"
        );
        ensure!(
            value.encode_utf16().count() <= 64,
            "Project collection names must not exceed 64 UTF-16 code units"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "Project collection names must not contain control characters"
        );
        ensure!(
            !value
                .chars()
                .any(|character| "\\/:*?\"<>;#${},+=[]|".contains(character)),
            "Project collection name contains a forbidden character"
        );
        ensure!(
            !value.starts_with('_') && !value.starts_with('.') && !value.ends_with('.'),
            "Project collection names must not start with an underscore or start/end with a period"
        );
        ensure!(
            !value.contains(".."),
            "Project collection names must not contain consecutive periods"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsProjectCollectionName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsProjectCollectionName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsProjectCollectionName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for AzureDevOpsProjectCollectionName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsProjectCollectionName {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureDevOpsProjectCollectionName {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsProjectCollectionName> for String {
    fn from(value: &AzureDevOpsProjectCollectionName) -> Self {
        value.as_str().to_owned()
    }
}

impl From<AzureDevOpsProjectCollectionName> for String {
    fn from(value: AzureDevOpsProjectCollectionName) -> Self {
        value.0.into()
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectCollectionName {
    fn arbitrary(unstructured: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        const CHARACTERS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_ ";
        let length = unstructured.int_in_range(1..=64)?;
        let mut value = String::with_capacity(length);
        value.push('C');
        for _ in 1..length {
            value.push(*unstructured.choose(CHARACTERS)? as char);
        }
        Self::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectCollectionName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectCollectionName);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_names_preserve_spaces_unicode_and_existing_default_scope() -> Result<()> {
        for value in ["My Collection", "Collection_1", "DefaultCollection", "集合"] {
            let name = AzureDevOpsProjectCollectionName::try_new(value)?;
            assert_eq!(name.as_str(), value);
        }
        assert!(AzureDevOpsProjectCollectionName::try_new("a".repeat(64)).is_ok());
        assert!(AzureDevOpsProjectCollectionName::try_new("😀".repeat(32)).is_ok());
        Ok(())
    }

    #[test]
    fn invalid_collection_spellings_are_rejected_during_construction() {
        for value in [
            "", " ", "a/b", "a\\b", "a\n", "_name", ".name", "name.", "a..b", "a?b",
        ] {
            assert!(AzureDevOpsProjectCollectionName::try_new(value).is_err());
        }
        assert!(AzureDevOpsProjectCollectionName::try_new("a".repeat(65)).is_err());
        assert!(AzureDevOpsProjectCollectionName::try_new("😀".repeat(33)).is_err());
    }
}
