use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The name of an Azure DevOps variable group.
///
/// Microsoft documentation: [VariableGroup](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#variablegroup).
/// This library rejects blank values and control characters as local safety
/// constraints, without imposing undocumented platform naming restrictions.
/// Exact spelling, spaces, Unicode and valid separators are preserved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsVariableGroupName(String);

impl AzureDevOpsVariableGroupName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "Variable group names must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "Variable group names must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsVariableGroupName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsVariableGroupName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsVariableGroupName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsVariableGroupName {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsVariableGroupName> for String {
    fn from(value: &AzureDevOpsVariableGroupName) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsVariableGroupName> for String {
    fn from(value: AzureDevOpsVariableGroupName) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsVariableGroupName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsVariableGroupName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?).unwrap_or_else(|_| {
            Self::try_new("Synthetic variable group").expect("valid synthetic value")
        }))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsVariableGroupName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsVariableGroupName);
