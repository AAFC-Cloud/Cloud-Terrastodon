use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The exact name of an Azure DevOps build definition.
///
/// See the `name` field in Microsoft's [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#builddefinitionreference).
/// This library rejects blank values and control characters as local safety
/// constraints. It preserves spaces, Unicode and exact spelling, and imposes
/// no speculative platform length or naming restrictions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildDefinitionName(String);

impl AzureDevOpsBuildDefinitionName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "Definition names must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "Definition names must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildDefinitionName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildDefinitionName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildDefinitionName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildDefinitionName {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildDefinitionName> for String {
    fn from(value: &AzureDevOpsBuildDefinitionName) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildDefinitionName> for String {
    fn from(value: AzureDevOpsBuildDefinitionName) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildDefinitionName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?).unwrap_or_else(|_| {
            Self::try_new("Synthetic pipeline").expect("valid synthetic value")
        }))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionName);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionName);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_definition_names_preserve_text_and_reject_invalid_values() {
        let text = "  Synthetic pipeline — CI  ";
        let value = AzureDevOpsBuildDefinitionName::try_new(text).unwrap();
        assert_eq!(value.as_str(), text);
        for invalid in ["", "  ", "\n", "synthetic\u{1b}"] {
            assert!(AzureDevOpsBuildDefinitionName::try_new(invalid).is_err());
        }
    }
}
