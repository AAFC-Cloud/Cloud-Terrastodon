use arbitrary::Arbitrary;
use std::ops::Deref;
use std::str::FromStr;

/// A slash-separated Azure DevOps work item query or folder path.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemQueryPath(String);

impl AzureDevOpsWorkItemQueryPath {
    pub fn try_new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        for segment in value.split('/') {
            if segment.trim().is_empty() || matches!(segment, "." | "..") {
                eyre::bail!("Query path contains an invalid segment");
            }
        }
        Ok(Self(value))
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryPath {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = String::arbitrary(u)?;
        Self::try_new(value)
            .or_else(|_| Self::try_new("Shared Queries"))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl Deref for AzureDevOpsWorkItemQueryPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AzureDevOpsWorkItemQueryPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemQueryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AzureDevOpsWorkItemQueryPath {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemQueryPath {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureDevOpsWorkItemQueryPath> for String {
    fn from(value: AzureDevOpsWorkItemQueryPath) -> Self {
        value.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryPath);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryPath);
