use arbitrary::Arbitrary;
use std::ops::Deref;
use std::str::FromStr;

/// A work item relation type reference name used by Azure DevOps.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemRelationTypeName(String);

impl AzureDevOpsWorkItemRelationTypeName {
    pub fn try_new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            eyre::bail!("Relation type name must not be empty");
        }
        if matches!(value.as_str(), "." | "..") {
            eyre::bail!("Invalid API path segment");
        }
        Ok(Self(value))
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemRelationTypeName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = String::arbitrary(u)?;
        Self::try_new(value)
            .or_else(|_| Self::try_new("System.LinkTypes.Related"))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl Deref for AzureDevOpsWorkItemRelationTypeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AzureDevOpsWorkItemRelationTypeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemRelationTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AzureDevOpsWorkItemRelationTypeName {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemRelationTypeName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureDevOpsWorkItemRelationTypeName> for String {
    fn from(value: AzureDevOpsWorkItemRelationTypeName) -> Self {
        value.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationTypeName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationTypeName);
