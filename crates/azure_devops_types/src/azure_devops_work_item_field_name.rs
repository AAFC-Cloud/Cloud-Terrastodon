use arbitrary::Arbitrary;
use std::borrow::Borrow;
use std::ops::Deref;
use std::str::FromStr;

/// A work item field reference name used in Azure DevOps API paths.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemFieldName(String);

impl AzureDevOpsWorkItemFieldName {
    pub fn try_new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            eyre::bail!("Field name must not be empty");
        }
        if matches!(value.as_str(), "." | "..") {
            eyre::bail!("Invalid API path segment");
        }
        Ok(Self(value))
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemFieldName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = String::arbitrary(u)?;
        Self::try_new(value)
            .or_else(|_| Self::try_new("Synthetic.Field"))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

impl Deref for AzureDevOpsWorkItemFieldName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AzureDevOpsWorkItemFieldName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for AzureDevOpsWorkItemFieldName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AzureDevOpsWorkItemFieldName {
    /// Escapes this field reference name for use as a JSON Patch path segment.
    pub fn json_patch_path(&self) -> String {
        format!("/fields/{}", self.replace('~', "~0").replace('/', "~1"))
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemFieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for AzureDevOpsWorkItemFieldName {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemFieldName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureDevOpsWorkItemFieldName> for String {
    fn from(value: AzureDevOpsWorkItemFieldName) -> Self {
        value.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemFieldName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemFieldName);
