use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;

/// A positive Azure DevOps build definition identifier.
///
/// See the `id` field in Microsoft's [BuildDefinitionReference schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#builddefinitionreference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = i32)]
pub struct AzureDevOpsBuildDefinitionId(i32);

impl AzureDevOpsBuildDefinitionId {
    pub fn new(value: i32) -> Result<Self> {
        ensure!(value > 0, "Build definition IDs must be positive integers");
        Ok(Self(value))
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for AzureDevOpsBuildDefinitionId {
    type Error = eyre::Report;

    fn try_from(value: i32) -> Result<Self> {
        Self::new(value)
    }
}

impl From<&AzureDevOpsBuildDefinitionId> for i32 {
    fn from(value: &AzureDevOpsBuildDefinitionId) -> Self {
        value.0
    }
}

impl FromStr for AzureDevOpsBuildDefinitionId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value.parse()?)
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildDefinitionId {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(1..=i32::MAX)?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionId);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_definition_identifiers_require_positive_numeric_values() {
        for invalid in ["0", "-1", "2147483648", "synthetic"] {
            assert!(invalid.parse::<AzureDevOpsBuildDefinitionId>().is_err());
        }
        for valid in [1, i32::MAX] {
            let id = AzureDevOpsBuildDefinitionId::new(valid).unwrap();
            assert_eq!(id.get(), valid);
        }
    }
}
