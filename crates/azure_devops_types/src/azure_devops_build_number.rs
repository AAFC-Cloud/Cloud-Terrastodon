use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The build number/name of a run, represented by the service as a string.
///
/// See the `buildNumber` field in Microsoft's [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#build).
/// This library rejects blank values and control characters as local safety
/// constraints. It preserves spaces, Unicode and exact spelling, and imposes
/// no speculative platform length or naming restrictions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildNumber(String);

impl AzureDevOpsBuildNumber {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(!value.trim().is_empty(), "Build numbers must not be blank");
        ensure!(
            !value.chars().any(char::is_control),
            "Build numbers must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildNumber {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildNumber {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildNumber {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildNumber {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildNumber> for String {
    fn from(value: &AzureDevOpsBuildNumber) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildNumber> for String {
    fn from(value: AzureDevOpsBuildNumber) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildNumber {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?)
            .unwrap_or_else(|_| Self::try_new("synthetic-1").expect("valid synthetic value")))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildNumber);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildNumber);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_numbers_preserve_string_identity_and_reject_invalid_values() {
        let text = "  00042 — release  ";
        let value = AzureDevOpsBuildNumber::try_new(text).unwrap();
        assert_eq!(value.as_str(), text);
        for invalid in ["", "  ", "\n", "synthetic\u{1b}"] {
            assert!(AzureDevOpsBuildNumber::try_new(invalid).is_err());
        }
    }
}
