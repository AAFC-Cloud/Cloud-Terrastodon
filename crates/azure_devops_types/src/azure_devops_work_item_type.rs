use arbitrary::Arbitrary;
use std::ops::Deref;
use std::str::FromStr;

/// A work item type name accepted by Azure DevOps work item APIs.
///
/// The constructor performs the lexical checks that used to be deferred until
/// a request was sent, so an `AzureDevOpsWorkItemType` cannot contain an empty
/// or traversal path segment.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemType(String);

impl AzureDevOpsWorkItemType {
    pub fn try_new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            eyre::bail!("Work item type must not be empty");
        }
        if matches!(value.as_str(), "." | "..") {
            eyre::bail!("Invalid API path segment");
        }
        Ok(Self(value))
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemType {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = String::arbitrary(u)?;
        Self::try_new(value)
            .or_else(|_| Self::try_new("Synthetic"))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl Deref for AzureDevOpsWorkItemType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AzureDevOpsWorkItemType {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AzureDevOpsWorkItemType {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemType {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureDevOpsWorkItemType> for String {
    fn from(value: AzureDevOpsWorkItemType) -> Self {
        value.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemType);
